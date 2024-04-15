use teloxide::{
    payloads::{EditMessageTextSetters, SendMessageSetters},
    prelude::*,
    types::{Message, MessageId, ParseMode},
};

use crate::bot::{
    dispatcher::State,
    handler::{
        constants::{COMMAND_HELP, NO_TEXT_MESSAGE},
        utils::{get_currency, make_keyboard, HandlerResult, UserDialogue},
    },
    processor::{get_chat_setting, set_chat_setting, update_chat_default_currency, ChatSetting},
};

use super::{
    constants::CURRENCY_DEFAULT,
    utils::{parse_time_zone, retrieve_time_zone},
};

/* Utilities */
const CANCEL_MESSAGE: &str = "Sure, no changes to the settings have been made! 👌";
const TIME_ZONE_DESCRIPTION: &str = "*Time Zone* — Time zone to display date and time";
const DEFAULT_CURRENCY_DESCRIPTION: &str =
    "*Default Currency* — Currency used by default if left blank";
const CURRENCY_CONVERSION_DESCRIPTION: &str =
    "*Currency Conversion* — Convert currencies when simplifying balances";

// Displays the first settings menu.
async fn display_settings_menu(
    bot: Bot,
    dialogue: UserDialogue,
    chat_id: String,
    msg_id: Option<MessageId>,
) -> HandlerResult {
    let buttons = vec![
        "Time Zone",
        "Default Currency",
        "Currency Conversion",
        "Cancel",
    ];
    let keyboard = make_keyboard(buttons, Some(2));
    let message = format!(
        "Of course\\! Here are the settings you can configure\\. What would you like to view or edit?\n\n{TIME_ZONE_DESCRIPTION}\n{DEFAULT_CURRENCY_DESCRIPTION}\n{CURRENCY_CONVERSION_DESCRIPTION}",
    );
    match msg_id {
        Some(id) => {
            bot.edit_message_text(chat_id, id, message)
                .parse_mode(ParseMode::MarkdownV2)
                .reply_markup(keyboard)
                .await?;
        }
        None => {
            bot.send_message(chat_id, message)
                .parse_mode(ParseMode::MarkdownV2)
                .reply_markup(keyboard)
                .await?;
        }
    }
    dialogue.update(State::SettingsMenu).await?;
    Ok(())
}

/* Handles a repeated call to edit/delete payment entry.
 * Does nothing, simply notifies the user.
 */
pub async fn handle_repeated_settings(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(
        msg.chat.id,
        "🚫 You are already checking out the settings! Please complete or cancel the current operation before starting a new one.",
        ).await?;
    Ok(())
}

/* Cancels the edit/delete payment operation.
 * Can be called at any step of the process.
 */
pub async fn cancel_settings(bot: Bot, dialogue: UserDialogue, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, CANCEL_MESSAGE).await?;
    dialogue.exit().await?;
    Ok(())
}

/* Blocks user command.
 * Called when user attempts to start another operation in the middle of editing/deleting a payment.
 */
pub async fn block_settings(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(
        msg.chat.id,
        "🚫 You are currently checking out the settings! Please complete or cancel the current payment entry before starting another command.",
        ).await?;
    Ok(())
}

/* Allows user to view and edit chat settings.
 * Bot presents a button menu of setting options.
 */
pub async fn action_settings(bot: Bot, dialogue: UserDialogue, msg: Message) -> HandlerResult {
    let chat_id = msg.chat.id.to_string();
    display_settings_menu(bot, dialogue, chat_id, None).await?;
    Ok(())
}

/* Handles the user's selection from the settings menu.
 * Bot receives a callback query from the user.
 */
pub async fn action_settings_menu(
    bot: Bot,
    dialogue: UserDialogue,
    query: CallbackQuery,
) -> HandlerResult {
    if let Some(button) = &query.data {
        bot.answer_callback_query(query.id.to_string()).await?;
        if let Some(msg) = query.message {
            let chat_id = msg.chat.id.to_string();
            match button.as_str() {
                "Time Zone" => {
                    let time_zone = retrieve_time_zone(&chat_id);
                    let buttons = vec!["Back", "Edit"];
                    let keyboard = make_keyboard(buttons, Some(2));
                    bot.edit_message_text(
                        chat_id,
                        msg.id,
                        format!(
                            "Time Zone: {}\n\nWould you like to edit the time zone for this chat?",
                            time_zone
                        ),
                    )
                    .reply_markup(keyboard)
                    .await?;
                    dialogue.update(State::SettingsTimeZoneMenu).await?;
                }
                "Default Currency" => {
                    let setting = get_chat_setting(&chat_id, ChatSetting::DefaultCurrency(None))?;
                    if let ChatSetting::DefaultCurrency(Some(currency)) = setting {
                        let currency_info: String;
                        let buttons: Vec<&str>;
                        if currency == CURRENCY_DEFAULT.0 {
                            currency_info = format!("Default Currency is NOT set.");
                            buttons = vec!["Back", "Edit"];
                        } else {
                            currency_info = format!("Default Currency: {}", currency);
                            buttons = vec!["Disable", "Edit", "Back"];
                        }
                        let keyboard = make_keyboard(buttons, Some(2));

                        bot.edit_message_text(
                            chat_id,
                            msg.id,
                            format!(
                                "{currency_info}\n\nWould you like to edit the default currency for this chat?",
                                )).reply_markup(keyboard)
                            .await?;
                        dialogue.update(State::SettingsDefaultCurrencyMenu).await?;
                    }
                }
                "Currency Conversion" => {
                    let setting =
                        get_chat_setting(&chat_id, ChatSetting::CurrencyConversion(None))?;
                    if let ChatSetting::CurrencyConversion(Some(convert)) = setting {
                        let status: &str;
                        let prompt: &str;
                        let buttons: Vec<&str>;
                        if convert {
                            status = "ENABLED";
                            buttons = vec!["Back", "Turn Off"];
                            prompt = "Do you wish to turn off currency conversion for this chat?";
                        } else {
                            let currency =
                                get_chat_setting(&chat_id, ChatSetting::DefaultCurrency(None))?;
                            if let ChatSetting::DefaultCurrency(Some(currency)) = currency {
                                if currency == CURRENCY_DEFAULT.0 {
                                    buttons = vec!["Back"];
                                    prompt = "To turn on currency conversion, you must first set a default currency for the chat!";
                                } else {
                                    buttons = vec!["Back", "Turn On"];
                                    prompt =
                                        "Do you wish to turn on currency conversion for this chat?";
                                }
                            } else {
                                // Should not occur, these are placeholder values
                                buttons = vec!["Back"];
                                prompt = "To turn on currency conversion, you must first set a default currency for the chat!";
                            }
                            status = "DISABLED";
                        }

                        let keyboard = make_keyboard(buttons.clone(), Some(buttons.len()));

                        bot.edit_message_text(
                            chat_id,
                            msg.id,
                            format!("Currency Conversion is currently {status}.\n\n{prompt}",),
                        )
                        .reply_markup(keyboard)
                        .await?;
                        dialogue.update(State::SettingsCurrencyConversion).await?;
                    }
                }
                "Cancel" => {
                    cancel_settings(bot, dialogue, msg).await?;
                }
                _ => {
                    if let Some(user) = msg.from() {
                        log::error!(
                            "Settings Menu - Invalid button for user {} in chat {}: {}",
                            user.id,
                            chat_id,
                            button
                        );
                    }
                }
            }
        }
    }
    Ok(())
}

/* Presents the time zone for the chat.
 * Receives a callback query on whether the user wants to edit the time zone.
 */
pub async fn action_time_zone_menu(
    bot: Bot,
    dialogue: UserDialogue,
    query: CallbackQuery,
) -> HandlerResult {
    if let Some(button) = &query.data {
        bot.answer_callback_query(query.id.to_string()).await?;
        if let Some(msg) = query.message {
            let chat_id = msg.chat.id;
            match button.as_str() {
                "Back" => {
                    display_settings_menu(bot, dialogue, chat_id.to_string(), Some(msg.id)).await?;
                }
                "Edit" => {
                    let time_zone = retrieve_time_zone(&chat_id.to_string());
                    bot.edit_message_text(
                            msg.chat.id,
                            msg.id,
                            format!(
                                "Time Zone: {}\n\nWhat time zone would you like to set for this chat? If you are unsure on which time zones are supported, check out the user guide with {COMMAND_HELP}!",
                                time_zone
                                ),
                                )
                            .await?;
                    dialogue.update(State::SettingsTimeZone).await?;
                }
                _ => {
                    if let Some(user) = msg.from() {
                        log::error!(
                            "Settings Time Zone Menu - Invalid button for user {} in chat {}: {}",
                            user.id,
                            chat_id,
                            button
                        );
                    }
                }
            }
        }
    }
    Ok(())
}

/* Sets the time zone for the chat.
 * Bot receives a string representing the time zone code, and calls processor.
 */
pub async fn action_settings_time_zone(
    bot: Bot,
    dialogue: UserDialogue,
    msg: Message,
) -> HandlerResult {
    let chat_id = msg.chat.id.to_string();
    match msg.text() {
        Some(text) => {
            let time_zone = parse_time_zone(text);
            match time_zone {
                Ok(time_zone) => {
                    let setting = ChatSetting::TimeZone(Some(text.to_string()));
                    set_chat_setting(&chat_id, setting)?;
                    bot.send_message(
                        msg.chat.id,
                        format!("The Time Zone has been set to {}! 👍", time_zone),
                    )
                    .await?;
                    dialogue.exit().await?;
                }
                Err(err) => {
                    bot.send_message(chat_id, format!("{}", err)).await?;
                }
            }
        }
        None => {
            bot.send_message(chat_id, format!("{NO_TEXT_MESSAGE}"))
                .await?;
        }
    }
    Ok(())
}

/* Presents the default currency for the chat.
 * Receives a callback query on whether the user wants to edit the default currency.
 */
pub async fn action_default_currency_menu(
    bot: Bot,
    dialogue: UserDialogue,
    query: CallbackQuery,
) -> HandlerResult {
    if let Some(button) = &query.data {
        bot.answer_callback_query(query.id.to_string()).await?;
        if let Some(msg) = query.message {
            let chat_id = msg.chat.id.to_string();
            match button.as_str() {
                "Disable" => {
                    update_chat_default_currency(&chat_id, CURRENCY_DEFAULT.0)?;
                    bot.send_message(
                        msg.chat.id,
                        format!("Default Currency has been disabled for future payments! 👍"),
                    )
                    .await?;
                    dialogue.exit().await?;
                }
                "Edit" => {
                    let setting = get_chat_setting(&chat_id, ChatSetting::DefaultCurrency(None))?;
                    if let ChatSetting::DefaultCurrency(Some(currency)) = setting {
                        let currency_info: String;
                        if currency == CURRENCY_DEFAULT.0 {
                            currency_info = format!("Default Currency is NOT set.");
                        } else {
                            currency_info = format!("Default Currency: {}", currency);
                        }

                        bot.edit_message_text(
                            chat_id,
                            msg.id,
                            format!(
                                "{currency_info}\n\nWhat would you like to set as the default currency? If you are unsure on which currencies are supported, check out the user guide with {COMMAND_HELP}!",
                                ))
                            .await?;
                        dialogue.update(State::SettingsDefaultCurrency).await?;
                    }
                }
                "Back" => {
                    display_settings_menu(bot, dialogue, chat_id, Some(msg.id)).await?;
                }
                _ => {
                    if let Some(user) = msg.from() {
                        log::error!(
                            "Settings Time Zone Menu - Invalid button for user {} in chat {}: {}",
                            user.id,
                            chat_id,
                            button
                        );
                    }
                }
            }
        }
    }
    Ok(())
}

/* Sets the default currency for the chat.
 * Bot receives a string representing the currency code, and calls processor.
 */
pub async fn action_settings_default_currency(
    bot: Bot,
    dialogue: UserDialogue,
    msg: Message,
) -> HandlerResult {
    let chat_id = msg.chat.id.to_string();
    match msg.text() {
        Some(text) => {
            let currency = get_currency(text);
            match currency {
                Ok(currency) => {
                    update_chat_default_currency(&chat_id, &currency.0)?;
                    bot.send_message(
                        msg.chat.id,
                        format!(
                            "Default Currency has been set to {} for future payments! 👍",
                            currency.0
                        ),
                    )
                    .await?;
                    dialogue.exit().await?;
                }
                Err(err) => {
                    bot.send_message(
                        chat_id,
                        format!(
                            "{} You can check out the supported currencies in the user guide with {COMMAND_HELP}.",
                            err
                            ))
                        .await?;
                }
            }
        }
        None => {
            bot.send_message(chat_id, format!("{NO_TEXT_MESSAGE}"))
                .await?;
        }
    }
    Ok(())
}

/* Sets whether currency conversion is enabled for the chat.
 * Bot receives a callback query, and calls processor.
 */
pub async fn action_settings_currency_conversion(
    bot: Bot,
    dialogue: UserDialogue,
    query: CallbackQuery,
) -> HandlerResult {
    if let Some(button) = &query.data {
        bot.answer_callback_query(query.id.to_string()).await?;
        if let Some(msg) = query.message {
            let chat_id = msg.chat.id.to_string();
            match button.as_str() {
                "Back" => {
                    display_settings_menu(bot, dialogue, chat_id, Some(msg.id)).await?;
                }
                "Turn On" => {
                    let setting = ChatSetting::CurrencyConversion(Some(true));
                    set_chat_setting(&chat_id, setting)?;
                    bot.edit_message_text(
                        msg.chat.id,
                        msg.id,
                        "Currency Conversion has been turned on for this chat! 👍",
                    )
                    .await?;
                    dialogue.exit().await?;
                }
                "Turn Off" => {
                    let setting = ChatSetting::CurrencyConversion(Some(false));
                    set_chat_setting(&chat_id, setting)?;
                    bot.edit_message_text(
                        msg.chat.id,
                        msg.id,
                        "Currency Conversion has been turned off for this chat! 👍",
                    )
                    .await?;
                    dialogue.exit().await?;
                }
                _ => {
                    if let Some(user) = msg.from() {
                        log::error!(
                            "Settings Menu - Invalid button for user {} in chat {}: {}",
                            user.id,
                            msg.chat.id,
                            button
                        );
                    }
                }
            }
        }
    }
    Ok(())
}
