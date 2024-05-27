use teloxide::{
    prelude::*,
    types::{Message, MessageId},
};

use crate::bot::{
    currency::{Currency, CURRENCY_DEFAULT},
    handler::{
        constants::{STATEMENT_INSTRUCTIONS_MESSAGE, UNKNOWN_ERROR_MESSAGE},
        utils::{
            display_amount, display_username, get_currency, make_keyboard,
            process_valid_currencies, send_bot_message, HandlerResult, UserDialogue,
        },
    },
    processor::{
        get_chat_setting, retrieve_spending_data, ChatSetting, SpendingData, UserSpending,
    },
    State,
};

use super::utils::{assert_handle_request_limit, StatementOption};

/* Utilities */

fn display_individual_spending(spending: UserSpending, currency: Currency) -> String {
    format!(
        "{}\n    Total Spent: {}\n    Total Paid For: {}\n",
        display_username(&spending.username),
        display_amount(spending.spending, currency.1),
        display_amount(spending.paid, currency.1)
    )
}

fn display_spendings(spending_data: &SpendingData) -> String {
    if spending_data.group_spending == 0 {
        return format!("Total Group Spending: 0\n");
    }

    let currency = match get_currency(&spending_data.currency) {
        Ok(currency) => currency,
        // Should not occur. Currency string is from database, so should exist.
        Err(_) => ("NIL".to_string(), 2),
    };

    let mut individual_spendings = String::new();
    for spending in &spending_data.user_spendings {
        individual_spendings.push_str(&display_individual_spending(
            spending.clone(),
            currency.clone(),
        ));
    }

    format!(
        "Total Group Spending: {}\n\n{}",
        display_amount(spending_data.group_spending, currency.1),
        individual_spendings
    )
}

async fn handle_spendings_with_option(
    bot: Bot,
    dialogue: UserDialogue,
    msg: Message,
    chat_id: String,
    sender_id: String,
    mut option: StatementOption,
    id: Option<MessageId>,
) -> HandlerResult {
    let spending_data = retrieve_spending_data(&chat_id, option.clone()).await;

    match spending_data {
        Ok(mut spending_data) => {
            let default_currency =
                match get_chat_setting(&chat_id, ChatSetting::DefaultCurrency(None)) {
                    Ok(ChatSetting::DefaultCurrency(Some(currency))) => currency,
                    _ => CURRENCY_DEFAULT.0.to_string(),
                };

            let mut valid_currencies = process_valid_currencies(
                &chat_id,
                &sender_id,
                option.clone(),
                default_currency.clone(),
            );

            // If no default currency, NIL has no balances, but other currencies do
            if spending_data.group_spending == 0 && valid_currencies.len() > 0 {
                let currency = valid_currencies.first().unwrap().clone();
                option = StatementOption::Currency(currency.clone());
                spending_data = match retrieve_spending_data(&chat_id, option.clone()).await {
                    Ok(new_data) => {
                        valid_currencies.retain(|curr| curr != &currency);
                        new_data
                    }
                    Err(_err) => spending_data,
                };
            }

            let ref_valid_currencies = valid_currencies
                .iter()
                .map(|x| x.as_str())
                .collect::<Vec<&str>>();

            let has_buttons = valid_currencies.len() > 0;
            let keyboard = make_keyboard(ref_valid_currencies, Some(2));

            let header = if let StatementOption::Currency(curr) = option {
                if curr == CURRENCY_DEFAULT.0 {
                    format!("🔥 Here are the total spendings!")
                } else {
                    format!("🔥 Here are the total spendings for {curr}!")
                }
            } else if has_buttons {
                format!("🔥 Here are the total spendings, converted to {default_currency}!")
            } else {
                format!("🔥 Here are the total spendings!")
            };

            match id {
                Some(id) => {
                    bot.edit_message_text(
                        chat_id.clone(),
                        id,
                        format!(
                            "{}\n\n{}\n{}",
                            header,
                            display_spendings(&spending_data),
                            if has_buttons {
                                STATEMENT_INSTRUCTIONS_MESSAGE
                            } else {
                                ""
                            }
                        ),
                    )
                    .reply_markup(keyboard)
                    .await?;
                }
                None => {
                    send_bot_message(
                        &bot,
                        &msg,
                        format!(
                            "{}\n\n{}\n{}",
                            header,
                            display_spendings(&spending_data),
                            if has_buttons {
                                STATEMENT_INSTRUCTIONS_MESSAGE
                            } else {
                                ""
                            }
                        ),
                    )
                    .reply_markup(keyboard)
                    .await?;
                }
            }
            dialogue.update(State::SpendingsMenu).await?;

            log::info!(
                "View Spendings - User {} viewed spendings for group {}: {}",
                sender_id,
                chat_id,
                display_spendings(&spending_data)
            );
        }
        Err(err) => {
            match id {
                Some(id) => {
                    bot.edit_message_text(chat_id.clone(), id, UNKNOWN_ERROR_MESSAGE)
                        .await?;
                }
                None => {
                    send_bot_message(&bot, &msg, UNKNOWN_ERROR_MESSAGE.to_string()).await?;
                }
            }
            log::error!(
                "View Spendings - User {} failed to view spendings for group {}: {}",
                sender_id,
                chat_id,
                err.to_string()
            );
        }
    }

    Ok(())
}

/* View the spendings for the group.
*/
pub async fn action_view_spendings(
    bot: Bot,
    dialogue: UserDialogue,
    msg: Message,
) -> HandlerResult {
    if !assert_handle_request_limit(msg.clone()) {
        return Ok(());
    }

    let chat_id = msg.chat.id.to_string();
    let sender_id = msg.from().as_ref().unwrap().id.to_string();
    let is_convert = match get_chat_setting(&chat_id, ChatSetting::CurrencyConversion(None)) {
        Ok(ChatSetting::CurrencyConversion(Some(value))) => value,
        _ => false,
    };
    let default_currency = match get_chat_setting(&chat_id, ChatSetting::DefaultCurrency(None)) {
        Ok(ChatSetting::DefaultCurrency(Some(currency))) => currency,
        _ => "NIL".to_string(),
    };

    let option = if is_convert {
        StatementOption::ConvertCurrency
    } else {
        StatementOption::Currency(default_currency.clone())
    };

    handle_spendings_with_option(bot, dialogue, msg, chat_id, sender_id, option, None).await?;

    Ok(())
}

/* Changes the display format of the spendings for the group.
 * Receives a callback query indicating new format change.
 */
pub async fn action_spendings_menu(
    bot: Bot,
    dialogue: UserDialogue,
    query: CallbackQuery,
) -> HandlerResult {
    if let Some(button) = &query.data {
        bot.answer_callback_query(query.id.to_string()).await?;
        let sender_id = query.from.id.to_string();

        if let Some(msg) = query.message {
            let chat_id = msg.chat.id.to_string();
            let id = msg.id;
            match button.as_str() {
                _ if button.as_str().starts_with("Convert To ") => {
                    let option = StatementOption::ConvertCurrency;
                    handle_spendings_with_option(
                        bot,
                        dialogue,
                        msg,
                        chat_id,
                        sender_id,
                        option,
                        Some(id),
                    )
                    .await?;
                }
                _ if button.as_str() == "No Currency" => {
                    let option = StatementOption::Currency(CURRENCY_DEFAULT.0.to_string());
                    handle_spendings_with_option(
                        bot,
                        dialogue,
                        msg,
                        chat_id,
                        sender_id,
                        option,
                        Some(id),
                    )
                    .await?;
                }
                _ if button.as_str().len() == 3 => {
                    let option = StatementOption::Currency(button.as_str().to_string());
                    handle_spendings_with_option(
                        bot,
                        dialogue,
                        msg,
                        chat_id,
                        sender_id,
                        option,
                        Some(id),
                    )
                    .await?;
                }
                _ => {
                    log::error!(
                        "View Spendings Menu - Invalid button in chat {} by user {}: {}",
                        chat_id,
                        sender_id,
                        button
                    );
                }
            }
        }
    }

    Ok(())
}
