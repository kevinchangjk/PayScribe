use teloxide::{
    prelude::*,
    types::{Message, MessageId},
};

use crate::bot::{
    currency::CURRENCY_DEFAULT,
    handler::{
        constants::UNKNOWN_ERROR_MESSAGE,
        utils::{display_balances, HandlerResult, StatementOption, UserDialogue},
    },
    processor::{get_chat_setting, retrieve_debts, retrieve_valid_currencies, ChatSetting},
    State,
};

use super::{constants::STATEMENT_INSTRUCTIONS_MESSAGE, utils::make_keyboard};

/* Utilities */

async fn handle_balances_with_option(
    bot: Bot,
    dialogue: UserDialogue,
    chat_id: String,
    sender_id: String,
    option: StatementOption,
    id: Option<MessageId>,
) -> HandlerResult {
    let balances_data = retrieve_debts(&chat_id, option.clone()).await;

    match balances_data {
        Ok(balances_data) => {
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

            if let StatementOption::Currency(ref curr) = option {
                valid_currencies.retain(|&x| x != curr);
            }

            // Add back default currency button if not NIL, and currently not default
            if default_currency != CURRENCY_DEFAULT.0 {
                if let StatementOption::Currency(ref curr) = option {
                    if curr != &default_currency {
                        valid_currencies.push(&default_currency);
                    }
                } else {
                    valid_currencies.push(&default_currency);
                }
            }

            // Special buttons
            let conversion_button = format!("Convert To {default_currency}");
            // Add conversion button only if not currently on convert, and have default currency
            if option != StatementOption::ConvertCurrency
                && default_currency != CURRENCY_DEFAULT.0
                && valid_currencies.len() > 0
            {
                valid_currencies.push(&conversion_button);
                // Add no currency button if no default currency, and not currently NIL
            } else if default_currency == CURRENCY_DEFAULT.0 {
                if let StatementOption::Currency(ref curr) = option {
                    if curr != CURRENCY_DEFAULT.0 {
                        valid_currencies.push("No Currency");
                    }
                }
            }

            let has_buttons = valid_currencies.len() > 0;
            let keyboard = make_keyboard(valid_currencies, Some(2));

            let header = if let StatementOption::Currency(curr) = option {
                if curr == CURRENCY_DEFAULT.0 {
                    format!("Here are the current balances!")
                } else {
                    format!("Here are the current {curr} balances!")
                }
            } else {
                format!("Here are the current balances, converted to {default_currency}!")
            };

            match id {
                Some(id) => {
                    bot.edit_message_text(
                        chat_id.clone(),
                        id,
                        format!(
                            "{}\n\n{}\n{}",
                            header,
                            display_balances(&balances_data),
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
                    bot.send_message(
                        chat_id.clone(),
                        format!(
                            "{}\n\n{}\n{}",
                            header,
                            display_balances(&balances_data),
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
            dialogue.update(State::BalancesMenu).await?;

            log::info!(
                "View Balances - User {} viewed balances for group {}",
                sender_id,
                chat_id
            );
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
                "View Balances - User {} failed to view balances for group {}: {}",
                sender_id,
                chat_id,
                err.to_string()
            );
        }
    }

    Ok(())
}

/* View the balances for the group.
*/
pub async fn action_view_balances(bot: Bot, dialogue: UserDialogue, msg: Message) -> HandlerResult {
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

    handle_balances_with_option(bot, dialogue, chat_id, sender_id, option, None).await?;

    Ok(())
}

/* Views the balances for the group.
 * Takes in a callback query representing the user option on format to display.
 */
pub async fn action_balances_menu(
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
                _ if button.as_str().starts_with("Convert To ") => {
                    let option = StatementOption::ConvertCurrency;
                    handle_balances_with_option(
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
                    let option = StatementOption::Currency(CURRENCY_DEFAULT.0.to_string());
                    handle_balances_with_option(
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
                    let option = StatementOption::Currency(button.as_str().to_string());
                    handle_balances_with_option(
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
                        "View Balances Menu - Invalid button in chat {} by user {}: {}",
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
