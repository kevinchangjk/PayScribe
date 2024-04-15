use teloxide::{
    payloads::SendMessageSetters,
    prelude::*,
    types::{Message, ParseMode},
};

use crate::bot::{
    dispatcher::State,
    handler::{
        constants::{COMMAND_CANCEL, COMMAND_HELP, NO_TEXT_MESSAGE},
        utils::{get_currency, make_keyboard, HandlerResult, UserDialogue},
    },
    processor::{get_chat_setting, set_chat_setting, ChatSetting},
};

use super::{constants::CURRENCY_DEFAULT, utils::parse_time_zone};

/* Utilities */
const CANCEL_MESSAGE: &str = "Sure, no changes to the settings have been made! 👌";
const TIME_ZONE_DESCRIPTION: &str = "*Time Zone* — Time zone to display date and time";
const DEFAULT_CURRENCY_DESCRIPTION: &str =
    "*Default Currency* — Currency used by default if left blank";
const CURRENCY_CONVERSION_DESCRIPTION: &str =
    "*Currency Conversion* — Convert currencies when simplifying balances";

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

async fn display_settings_menu(bot: Bot, dialogue: UserDialogue, chat_id: String) -> HandlerResult {
    let buttons = vec![
        "Time Zone",
        "Default Currency",
        "Currency Conversion",
        "Cancel",
    ];
    let keyboard = make_keyboard(buttons, Some(2));
    bot.send_message(chat_id, format!("Of course\\! Here are the settings you can configure\\. What would you like to view or edit?\n\n{TIME_ZONE_DESCRIPTION}\n{DEFAULT_CURRENCY_DESCRIPTION}\n{CURRENCY_CONVERSION_DESCRIPTION}"))
        .parse_mode(ParseMode::MarkdownV2)
        .reply_markup(keyboard)
        .await?;
    dialogue.update(State::SettingsMenu).await?;
    Ok(())
}

/* Allows user to view and edit chat settings.
 * Bot presents a button menu of setting options.
 */
pub async fn action_settings(bot: Bot, dialogue: UserDialogue, msg: Message) -> HandlerResult {
    let chat_id = msg.chat.id.to_string();
    display_settings_menu(bot, dialogue, chat_id).await?;
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
            match button.as_str() {
                "Time Zone" => {
                    let setting =
                        get_chat_setting(&msg.chat.id.to_string(), ChatSetting::TimeZone(None))?;
                    if let ChatSetting::TimeZone(Some(time_zone)) = setting {
                        bot.edit_message_text(
                            msg.chat.id,
                            msg.id,
                            format!(
                                "Current Time Zone: {}\n\nIf you wish to change the time zone, reply with the new time zone code. Otherwise, if you are happy with this, you can /cancel this operation.",
                                time_zone
                                ),
                                )
                            .await?;
                        dialogue.update(State::SettingsTimeZone).await?;
                    }
                }
                "Default Currency" => {
                    let setting = get_chat_setting(
                        &msg.chat.id.to_string(),
                        ChatSetting::DefaultCurrency(None),
                    )?;
                    if let ChatSetting::DefaultCurrency(Some(currency)) = setting {
                        let currency_info: String;
                        if currency == CURRENCY_DEFAULT.0 {
                            currency_info = format!("Default Currency is NOT set.");
                        } else {
                            currency_info = format!("Default Currency: {}", currency);
                        }
                        bot.edit_message_text(
                            msg.chat.id,
                            msg.id,
                            format!(
                                "{currency_info}\n\nIf you wish to change the default currency, reply with the new currency code (check out the documentation in {COMMAND_HELP} if unsure!).\n\nYou can also disable Default Currency by replying with \"NIL\". Or, to keep it as it is, {COMMAND_CANCEL} this operation.",
                                ),
                                )
                            .await?;
                        dialogue.update(State::SettingsDefaultCurrency).await?;
                    }
                }
                "Currency Conversion" => {
                    let setting = get_chat_setting(
                        &msg.chat.id.to_string(),
                        ChatSetting::CurrencyConversion(None),
                    )?;
                    if let ChatSetting::CurrencyConversion(Some(convert)) = setting {
                        let status: &str;
                        let action: &str;
                        let button: &str;
                        if convert {
                            status = "ENABLED";
                            action = "turn off";
                            button = "Turn Off";
                        } else {
                            status = "DISABLED";
                            action = "turn on";
                            button = "Turn On";
                        }

                        let buttons = vec!["Back", button];
                        let keyboard = make_keyboard(buttons, Some(2));

                        bot.edit_message_text(
                            msg.chat.id,
                            msg.id,
                            format!(
                                "Currency Conversion is currently {}.\n\nDo you wish to {} currency conversion for this chat?",
                                status,
                                action
                                ),
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

/* Sets the default currency for the chat.
 * Bot receives a string representing the currency code, and calls processor.
 */
pub async fn action_settings_default_currency(
    bot: Bot,
    dialogue: UserDialogue,
    msg: Message,
) -> HandlerResult {
    match msg.text() {
        Some(text) => {
            let currency = get_currency(text);
            match currency {
                Ok(currency) => {
                    let response: String;
                    if currency.0 == CURRENCY_DEFAULT.0 {
                        response = format!("The Default Currency has now been disabled! 👍");
                    } else {
                        response =
                            format!("The Default Currency has been set to {}! 👍", currency.0);
                    }
                    let setting = ChatSetting::DefaultCurrency(Some(currency.0.clone()));
                    set_chat_setting(&msg.chat.id.to_string(), setting)?;
                    bot.send_message(msg.chat.id, response).await?;
                    dialogue.exit().await?;
                }
                Err(err) => {
                    bot.send_message(
                        msg.chat.id,
                        format!(
                            "{} You can check out the supported currencies in the documentation with {COMMAND_HELP}.",
                            err
                        ),
                    )
                    .await?;
                }
            }
        }
        None => {
            bot.send_message(msg.chat.id, format!("{NO_TEXT_MESSAGE}"))
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
                    display_settings_menu(bot, dialogue, chat_id).await?;
                }
                "Turn On" => {
                    let setting = ChatSetting::CurrencyConversion(Some(true));
                    set_chat_setting(&chat_id, setting)?;
                    bot.send_message(
                        msg.chat.id,
                        "Currency Conversion has been turned on for this chat! 👍",
                    )
                    .await?;
                    dialogue.exit().await?;
                }
                "Turn Off" => {
                    let setting = ChatSetting::CurrencyConversion(Some(false));
                    set_chat_setting(&chat_id, setting)?;
                    bot.send_message(
                        msg.chat.id,
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
