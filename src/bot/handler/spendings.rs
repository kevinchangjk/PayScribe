use teloxide::{
    prelude::*,
    types::{Message, MessageId},
};

use crate::bot::{
    currency::{Currency, CURRENCY_DEFAULT},
    handler::{
        constants::{SPENDINGS_INSTRUCTIONS_MESSAGE, UNKNOWN_ERROR_MESSAGE},
        utils::{
            display_amount, display_username, get_currency, make_keyboard, HandlerResult,
            UserDialogue,
        },
    },
    processor::{
        get_chat_setting, retrieve_spending_data, retrieve_valid_currencies, ChatSetting,
        SpendingData, UserSpending,
    },
    State,
};

/* Utilities */

#[derive(PartialEq, Debug, Clone)]
pub enum SpendingsOption {
    Currency(String),
    ConvertCurrency,
}

fn display_individual_spending(spending: UserSpending, currency: Currency) -> String {
    format!(
        "{}\n    Total Spent: {}\n    Total Paid For: {}\n",
        display_username(&spending.username),
        display_amount(spending.spending, currency.1),
        display_amount(spending.paid, currency.1)
    )
}

fn display_spendings(spending_data: SpendingData) -> String {
    if spending_data.group_spending == 0 {
        return format!("Total Group Spending: 0");
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
        "Total Group Spending: {}\nTotal Individual Spendings:\n\n{}",
        display_amount(spending_data.group_spending, currency.1),
        individual_spendings
    )
}

async fn handle_spendings_with_option(
    bot: Bot,
    dialogue: UserDialogue,
    chat_id: String,
    sender_id: String,
    option: SpendingsOption,
    id: Option<MessageId>,
) -> HandlerResult {
    let spending_data = retrieve_spending_data(&chat_id, option.clone()).await;

    match spending_data {
        Ok(spending_data) => {
            let valid_currencies = match retrieve_valid_currencies(&chat_id) {
                Ok(currencies) => currencies,
                Err(_) => {
                    log::error!(
                        "View Spendings - User {} failed to retrieve valid currencies for group {}",
                        sender_id,
                        chat_id
                    );
                    vec![]
                }
            };

            let default_currency =
                match get_chat_setting(&chat_id, ChatSetting::DefaultCurrency(None)) {
                    Ok(ChatSetting::DefaultCurrency(Some(currency))) => currency,
                    _ => CURRENCY_DEFAULT.0.to_string(),
                };

            let mut valid_currencies: Vec<&str> =
                valid_currencies.iter().map(|s| s.as_ref()).collect();
            valid_currencies.retain(|&x| x != CURRENCY_DEFAULT.0 && x != default_currency);

            if let SpendingsOption::Currency(ref curr) = option {
                valid_currencies.retain(|&x| x != curr);
            }

            // Add back default currency button if not NIL, and currently not default
            if default_currency != CURRENCY_DEFAULT.0 {
                if let SpendingsOption::Currency(ref curr) = option {
                    if curr != &default_currency {
                        valid_currencies.push(&default_currency);
                    }
                } else {
                    valid_currencies.push(&default_currency);
                }
            }

            // Special buttons
            let conversion_button = format!("Convert to {default_currency}");
            // Add conversion button only if not currently on convert, and have default currency
            if option != SpendingsOption::ConvertCurrency
                && default_currency != CURRENCY_DEFAULT.0
                && valid_currencies.len() > 0
            {
                valid_currencies.push(&conversion_button);
                // Add no currency button if no default currency, and not currently NIL
            } else if default_currency == CURRENCY_DEFAULT.0 {
                if let SpendingsOption::Currency(ref curr) = option {
                    if curr != CURRENCY_DEFAULT.0 {
                        valid_currencies.push("No Currency");
                    }
                }
            }

            let has_buttons = valid_currencies.len() > 0;
            let keyboard = make_keyboard(valid_currencies, Some(2));

            let header = if let SpendingsOption::Currency(curr) = option {
                if curr == CURRENCY_DEFAULT.0 {
                    format!("Here are the total spendings!")
                } else {
                    format!("Here are the total spendings for {curr}!")
                }
            } else {
                format!("Here are the total spendings, converted to {default_currency}!")
            };

            match id {
                Some(id) => {
                    bot.edit_message_text(
                        chat_id,
                        id,
                        format!(
                            "{}\n\n{}\n{}",
                            header,
                            display_spendings(spending_data),
                            if has_buttons {
                                SPENDINGS_INSTRUCTIONS_MESSAGE
                            } else {
                                ""
                            }
                        ),
                    )
                    .reply_markup(keyboard)
                    .await?;
                }
                None => {
                    bot.send_message(
                        chat_id,
                        format!(
                            "{}\n\n{}\n{}",
                            header,
                            display_spendings(spending_data),
                            if has_buttons {
                                SPENDINGS_INSTRUCTIONS_MESSAGE
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
        }
        Err(err) => {
            match id {
                Some(id) => {
                    bot.edit_message_text(chat_id.clone(), id, UNKNOWN_ERROR_MESSAGE)
                        .await?;
                }
                None => {
                    bot.send_message(chat_id.clone(), UNKNOWN_ERROR_MESSAGE)
                        .await?;
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
        SpendingsOption::ConvertCurrency
    } else {
        SpendingsOption::Currency(default_currency.clone())
    };

    handle_spendings_with_option(bot, dialogue, chat_id, sender_id, option, None).await?;

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

        if let Some(Message { id, chat, .. }) = query.message {
            let chat_id = chat.id.to_string();
            match button.as_str() {
                _ if button.as_str().starts_with("Convert to ") => {
                    let option = SpendingsOption::ConvertCurrency;
                    handle_spendings_with_option(
                        bot,
                        dialogue,
                        chat_id,
                        sender_id,
                        option,
                        Some(id),
                    )
                    .await?;
                }
                _ if button.as_str() == "No Currency" => {
                    let option = SpendingsOption::Currency(CURRENCY_DEFAULT.0.to_string());
                    handle_spendings_with_option(
                        bot,
                        dialogue,
                        chat_id,
                        sender_id,
                        option,
                        Some(id),
                    )
                    .await?;
                }
                _ if button.as_str().len() == 3 => {
                    let option = SpendingsOption::Currency(button.as_str().to_string());
                    handle_spendings_with_option(
                        bot,
                        dialogue,
                        chat_id,
                        sender_id,
                        option,
                        Some(id),
                    )
                    .await?;
                }
                _ => {
                    log::error!(
                        "View Payments Menu - Invalid button in chat {} by user {}: {}",
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
