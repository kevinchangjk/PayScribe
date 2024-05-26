use teloxide::{
    payloads::{EditMessageTextSetters, SendMessageSetters},
    prelude::*,
    types::{Message, MessageId, ParseMode},
};

use crate::bot::{
    currency::CURRENCY_DEFAULT,
    dispatcher::State,
    handler::{
        constants::{
            COMMAND_CANCEL, CURRENCY_INSTRUCTIONS_MESSAGE, NO_TEXT_MESSAGE,
            TIME_ZONE_INSTRUCTIONS_MESSAGE,
        },
        utils::{
            get_currency, make_keyboard, parse_time_zone, retrieve_time_zone, HandlerResult,
            UserDialogue,
        },
    },
    processor::{get_chat_setting, set_chat_setting, update_chat_default_currency, ChatSetting},
};

use super::{
    constants::UNKNOWN_ERROR_MESSAGE,
    utils::{assert_handle_request_limit, delete_bot_messages, send_bot_message},
};

/* Utilities */
const CANCEL_MESSAGE: &str = "Okay! No changes to my settings have been made! 🌟";
const TIME_ZONE_DESCRIPTION: &str = "*🕔 Time Zone* — Time zone for displaying date and time";
const DEFAULT_CURRENCY_DESCRIPTION: &str = "💵 *Default Currency* — Currency used if left blank";
const CURRENCY_CONVERSION_DESCRIPTION: &str =
    "↔️ *Currency Conversion* — Convert currencies when calculating balances and spendings";

// Controls the state for misc handler actions that return to same state.
async fn repeat_state(dialogue: UserDialogue, new_message: MessageId) {
    let state = dialogue.get().await;
    match state {
        Ok(Some(State::SettingsMenu { mut messages })) => {
            messages.push(new_message);
            dialogue.update(State::SettingsMenu { messages });
        }
        Ok(Some(State::SettingsTimeZoneMenu { mut messages })) => {
            messages.push(new_message);
            dialogue.update(State::SettingsTimeZoneMenu { messages });
        }
        Ok(Some(State::SettingsTimeZone { mut messages })) => {
            messages.push(new_message);
            dialogue.update(State::SettingsTimeZone { messages });
        }
        Ok(Some(State::SettingsDefaultCurrencyMenu { mut messages })) => {
            messages.push(new_message);
            dialogue.update(State::SettingsDefaultCurrencyMenu { messages });
        }
        Ok(Some(State::SettingsDefaultCurrency { mut messages })) => {
            messages.push(new_message);
            dialogue.update(State::SettingsDefaultCurrency { messages });
        }
        Ok(Some(State::SettingsCurrencyConversion { mut messages })) => {
            messages.push(new_message);
            dialogue.update(State::SettingsCurrencyConversion { messages });
        }
        _ => (),
    }
}

// Displays the first settings menu.
async fn display_settings_menu(
    bot: &Bot,
    dialogue: &UserDialogue,
    msg: &Message,
    msg_id: Option<MessageId>,
    mut messages: Vec<MessageId>,
) -> HandlerResult {
    let buttons = vec!["🕔", "💵", "↔️", "Cancel"];
    let keyboard = make_keyboard(buttons, Some(2));
    let message = format!(
        "With pleasure\\! 😉 Let's see, here are the ⚙️ settings you can customize\\. What would you like to view or edit?\n\n{TIME_ZONE_DESCRIPTION}\n\n{DEFAULT_CURRENCY_DESCRIPTION}\n\n{CURRENCY_CONVERSION_DESCRIPTION}",
        );
    match msg_id {
        Some(id) => {
            bot.edit_message_text(msg.chat.id, id, message)
                .parse_mode(ParseMode::MarkdownV2)
                .reply_markup(keyboard)
                .await?;
            dialogue.update(State::SettingsMenu { messages }).await?;
        }
        None => {
            let new_message = send_bot_message(&bot, &msg, message)
                .parse_mode(ParseMode::MarkdownV2)
                .reply_markup(keyboard)
                .await?
                .id;
            messages.push(new_message);
            dialogue.update(State::SettingsMenu { messages }).await?;
        }
    }
    Ok(())
}

/* Handles a repeated call to edit/delete payment entry.
 * Does nothing, simply notifies the user.
 */
pub async fn handle_repeated_settings(
    bot: Bot,
    dialogue: UserDialogue,
    msg: Message,
) -> HandlerResult {
    if !assert_handle_request_limit(msg.clone()) {
        return Ok(());
    }

    let new_message = send_bot_message(
        &bot,
        &msg,
        format!("🚫 Oops! It seems like you're already in the middle of customizing my settings! Please finish or {COMMAND_CANCEL} this before starting another one with me."),
        ).await?.id;

    repeat_state(dialogue, new_message).await;

    Ok(())
}

/* Cancels the edit/delete payment operation.
 * Can be called at any step of the process.
 */
pub async fn cancel_settings(bot: Bot, dialogue: UserDialogue, msg: Message) -> HandlerResult {
    if !assert_handle_request_limit(msg.clone()) {
        return Ok(());
    }

    send_bot_message(&bot, &msg, CANCEL_MESSAGE.to_string())
        .await?
        .id;

    let state = dialogue.get().await;
    if let Ok(Some(state)) = state {
        match state {
            State::SettingsMenu { messages }
            | State::SettingsTimeZoneMenu { messages }
            | State::SettingsTimeZone { messages }
            | State::SettingsDefaultCurrencyMenu { messages }
            | State::SettingsDefaultCurrency { messages }
            | State::SettingsCurrencyConversion { messages } => {
                delete_bot_messages(&bot, &msg.chat.id.to_string(), messages).await;
            }
            _ => (),
        }
    }

    dialogue.exit().await?;

    Ok(())
}

/* Blocks user command.
 * Called when user attempts to start another operation in the middle of editing/deleting a payment.
 */
pub async fn block_settings(bot: Bot, dialogue: UserDialogue, msg: Message) -> HandlerResult {
    if !assert_handle_request_limit(msg.clone()) {
        return Ok(());
    }

    let new_message = send_bot_message(
        &bot,
        &msg,
        format!("🚫 Oops! It seems like you're in the middle of customizing my settings! Please finish or {COMMAND_CANCEL} this before starting something new with me."),
        ).await?.id;

    repeat_state(dialogue, new_message).await;

    Ok(())
}

/* Allows user to view and edit chat settings.
 * Bot presents a button menu of setting options.
 */
pub async fn action_settings(bot: Bot, dialogue: UserDialogue, msg: Message) -> HandlerResult {
    if !assert_handle_request_limit(msg.clone()) {
        return Ok(());
    }

    display_settings_menu(&bot, &dialogue, &msg, None, Vec::new()).await?;
    Ok(())
}

/* Handles the user's selection from the settings menu.
 * Bot receives a callback query from the user.
 */
pub async fn action_settings_menu(
    bot: Bot,
    dialogue: UserDialogue,
    query: CallbackQuery,
    messages: Vec<MessageId>,
) -> HandlerResult {
    if let Some(button) = &query.data {
        bot.answer_callback_query(query.id.to_string()).await?;
        if let Some(msg) = query.message {
            let chat_id = msg.chat.id.to_string();
            match button.as_str() {
                "🕔" => {
                    let time_zone = retrieve_time_zone(&chat_id);
                    let buttons = vec!["Back", "Edit"];
                    let keyboard = make_keyboard(buttons, Some(2));
                    bot.edit_message_text(
                        chat_id,
                        msg.id,
                        format!(
                            "🕔 Time Zone: {}\n\nWould you like to edit the time zone for this chat?",
                            time_zone
                        ),
                    )
                    .reply_markup(keyboard)
                    .await?;
                    dialogue
                        .update(State::SettingsTimeZoneMenu { messages })
                        .await?;
                }
                "💵" => {
                    let setting = get_chat_setting(&chat_id, ChatSetting::DefaultCurrency(None))?;
                    if let ChatSetting::DefaultCurrency(Some(currency)) = setting {
                        let currency_info: String;
                        let buttons: Vec<&str>;
                        if currency == CURRENCY_DEFAULT.0 {
                            currency_info = format!("💵 Default Currency is NOT set.");
                            buttons = vec!["Back", "Edit"];
                        } else {
                            currency_info = format!("💵 Default Currency: {}", currency);
                            buttons = vec!["Disable", "Edit", "Back"];
                        }
                        let keyboard = make_keyboard(buttons, Some(2));

                        bot.edit_message_text(
                            chat_id,
                            msg.id,
                            format!(
                                "{currency_info}\n\nWould you like to edit the default currency for this chat?",
                                ))
                            .reply_markup(keyboard)
                            .await?;
                        dialogue
                            .update(State::SettingsDefaultCurrencyMenu { messages })
                            .await?;
                    }
                }
                "↔️" => {
                    let setting =
                        get_chat_setting(&chat_id, ChatSetting::CurrencyConversion(None))?;
                    if let ChatSetting::CurrencyConversion(Some(convert)) = setting {
                        let status: &str;
                        let prompt: &str;
                        let buttons: Vec<&str>;
                        if convert {
                            status = "ENABLED ✅";
                            buttons = vec!["Back", "Turn Off"];
                            prompt =
                                "Would you like to turn off currency conversion for this chat?";
                        } else {
                            let currency =
                                get_chat_setting(&chat_id, ChatSetting::DefaultCurrency(None))?;
                            if let ChatSetting::DefaultCurrency(Some(currency)) = currency {
                                if currency == CURRENCY_DEFAULT.0 {
                                    buttons = vec!["Back"];
                                    prompt = "⭐️ If you wish to turn on currency conversion, please set a default currency first!";
                                } else {
                                    buttons = vec!["Back", "Turn On"];
                                    prompt =
                                        "Would you like to turn on currency conversion for this chat?";
                                }
                            } else {
                                // Should not occur, these are placeholder values
                                buttons = vec!["Back"];
                                prompt = "⭐️ If you wish to turn on currency conversion, please set a default currency first!";
                            }
                            status = "DISABLED ❌";
                        }

                        let keyboard = make_keyboard(buttons.clone(), Some(buttons.len()));

                        bot.edit_message_text(
                            chat_id,
                            msg.id,
                            format!("↔️ Currency Conversion is currently {status}.\n\n{prompt}",),
                        )
                        .reply_markup(keyboard)
                        .await?;
                        dialogue
                            .update(State::SettingsCurrencyConversion { messages })
                            .await?;
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
    messages: Vec<MessageId>,
) -> HandlerResult {
    if let Some(button) = &query.data {
        bot.answer_callback_query(query.id.to_string()).await?;
        if let Some(msg) = query.message {
            let chat_id = msg.chat.id;
            match button.as_str() {
                "Back" => {
                    display_settings_menu(&bot, &dialogue, &msg, Some(msg.id), messages).await?;
                }
                "Edit" => {
                    let time_zone = retrieve_time_zone(&chat_id.to_string());
                    bot.edit_message_text(
                        msg.chat.id,
                        msg.id,
                        format!(
                            "🕔 Time Zone: {}\n\nWhat time zone would you like to set?\n\n{TIME_ZONE_INSTRUCTIONS_MESSAGE}",
                            time_zone
                            ),
                            )
                        .await?;
                    dialogue
                        .update(State::SettingsTimeZone { messages })
                        .await?;
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
    messages: Vec<MessageId>,
) -> HandlerResult {
    let chat_id = msg.chat.id.to_string();
    match msg.text() {
        Some(text) => {
            let time_zone = parse_time_zone(text);
            match time_zone {
                Ok(time_zone) => {
                    let setting = ChatSetting::TimeZone(Some(text.to_string()));
                    let process = set_chat_setting(&chat_id, setting).await;
                    match process {
                        Ok(_) => {
                            send_bot_message(
                                &bot,
                                &msg,
                                format!("You got it! I've set the 🕔 Time Zone to {}!", time_zone),
                            )
                            .await?;

                            // Logging
                            log::info!(
                                "Settings Time Zone - Time zone set for chat {}: {}",
                                chat_id,
                                time_zone
                            );
                        }
                        Err(err) => {
                            send_bot_message(&bot, &msg, UNKNOWN_ERROR_MESSAGE.to_string()).await?;

                            // Logging
                            log::error!(
                                "Settings Time Zone - Error setting time zone for chat {}: {}",
                                chat_id,
                                err.to_string()
                            );
                        }
                    }
                    delete_bot_messages(&bot, &chat_id, messages).await?;
                    dialogue.exit().await?;
                }
                Err(err) => {
                    send_bot_message(&bot, &msg, err.to_string()).await?;
                }
            }
        }
        None => {
            let new_message = send_bot_message(&bot, &msg, format!("{NO_TEXT_MESSAGE}"))
                .await?
                .id;
            repeat_state(dialogue, new_message).await;
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
    messages: Vec<MessageId>,
) -> HandlerResult {
    if let Some(button) = &query.data {
        bot.answer_callback_query(query.id.to_string()).await?;
        if let Some(msg) = query.message {
            let chat_id = msg.chat.id.to_string();
            match button.as_str() {
                "Disable" => {
                    let process = update_chat_default_currency(&chat_id, CURRENCY_DEFAULT.0).await;
                    match process {
                        Ok(_) => {
                            send_bot_message(
                                &bot,
                                &msg,
                                format!("You got it! I've disabled the 💵 Default Currency!"),
                            )
                            .await?;

                            // Logging
                            log::info!(
                                "Settings Default Currency - Default currency disabled for chat {}",
                                chat_id
                            );
                        }
                        Err(err) => {
                            send_bot_message(&bot, &msg, UNKNOWN_ERROR_MESSAGE.to_string()).await?;

                            // Logging
                            log::error!(
                                "Settings Default Currency - Error setting default currency for chat {}: {}",
                                chat_id,
                                err.to_string()
                                );
                        }
                    }
                    delete_bot_messages(&bot, &chat_id, messages).await?;
                    dialogue.exit().await?;

                    // Logging
                    log::info!(
                        "Settings Default Currency - Default currency disabled for chat {}",
                        chat_id
                    );
                }
                "Edit" => {
                    let setting = get_chat_setting(&chat_id, ChatSetting::DefaultCurrency(None))?;
                    if let ChatSetting::DefaultCurrency(Some(currency)) = setting {
                        let currency_info: String;
                        if currency == CURRENCY_DEFAULT.0 {
                            currency_info = format!("💵 Default Currency is NOT set.");
                        } else {
                            currency_info = format!("💵 Default Currency: {}", currency);
                        }

                        bot.edit_message_text(
                            chat_id,
                            msg.id,
                            format!(
                                "{currency_info}\n\nWhat would you like to set as the default currency?\n\n{CURRENCY_INSTRUCTIONS_MESSAGE}",
                                ))
                            .await?;
                        dialogue
                            .update(State::SettingsDefaultCurrency { messages })
                            .await?;
                    }
                }
                "Back" => {
                    display_settings_menu(&bot, &dialogue, &msg, Some(msg.id), messages).await?;
                }
                _ => {
                    if let Some(user) = msg.from() {
                        log::error!(
                            "Settings Default Currency Menu - Invalid button for user {} in chat {}: {}",
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
    messages: Vec<MessageId>,
) -> HandlerResult {
    let chat_id = msg.chat.id.to_string();
    match msg.text() {
        Some(text) => {
            let currency = get_currency(text);
            match currency {
                Ok(currency) => {
                    let process = update_chat_default_currency(&chat_id, &currency.0).await;
                    match process {
                        Ok(_) => {
                            send_bot_message(
                                &bot,
                                &msg,
                                format!(
                                    "You got it! I've set the 💵 Default Currency to {}!",
                                    currency.0
                                ),
                            )
                            .await?;

                            // Logging
                            log::info!(
                                "Settings Default Currency - Default currency set for chat {}: {}",
                                chat_id,
                                currency.0
                            );
                        }
                        Err(err) => {
                            send_bot_message(&bot, &msg, UNKNOWN_ERROR_MESSAGE.to_string()).await?;

                            // Logging
                            log::error!(
                                "Settings Default Currency - Error setting default currency for chat {}: {}",
                                chat_id,
                                err.to_string()
                                );
                        }
                    }
                    delete_bot_messages(&bot, &chat_id, messages).await?;
                    dialogue.exit().await?;
                }
                Err(err) => {
                    send_bot_message(
                        &bot,
                        &msg,
                        format!("{}\n\n{CURRENCY_INSTRUCTIONS_MESSAGE}", err),
                    )
                    .await?;
                }
            }
        }
        None => {
            let new_message = send_bot_message(&bot, &msg, format!("{NO_TEXT_MESSAGE}"))
                .await?
                .id;
            repeat_state(dialogue, new_message).await;
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
    messages: Vec<MessageId>,
) -> HandlerResult {
    if let Some(button) = &query.data {
        bot.answer_callback_query(query.id.to_string()).await?;
        if let Some(msg) = query.message {
            let chat_id = msg.chat.id.to_string();
            match button.as_str() {
                "Back" => {
                    display_settings_menu(&bot, &dialogue, &msg, Some(msg.id), messages).await?;
                }
                "Turn On" => {
                    let setting = ChatSetting::CurrencyConversion(Some(true));
                    let process = set_chat_setting(&chat_id, setting).await;
                    match process {
                        Ok(_) => {
                            send_bot_message(
                                &bot,
                                &msg,
                                "You got it! I've turned on ↔️ Currency Conversion!".to_string(),
                            )
                            .await?;

                            // Logging
                            log::info!(
                                "Settings Currency Conversion - Currency conversion enabled for chat {}",
                                chat_id
                                );
                        }
                        Err(err) => {
                            send_bot_message(&bot, &msg, UNKNOWN_ERROR_MESSAGE.to_string()).await?;

                            // Logging
                            log::error!(
                                "Settings Currency Conversion - Error setting currency conversion for chat {}: {}",
                                chat_id,
                                err.to_string()
                                );
                        }
                    }
                    delete_bot_messages(&bot, &chat_id, messages).await?;
                    dialogue.exit().await?;
                }
                "Turn Off" => {
                    let setting = ChatSetting::CurrencyConversion(Some(false));
                    let process = set_chat_setting(&chat_id, setting).await;
                    match process {
                        Ok(_) => {
                            send_bot_message(
                                &bot,
                                &msg,
                                "You got it! I've turned off ↔️ Currency Conversion!".to_string(),
                            )
                            .await?;

                            // Logging
                            log::info!(
                                "Settings Currency Conversion - Currency conversion disabled for chat {}",
                                chat_id
                                );
                        }
                        Err(err) => {
                            send_bot_message(&bot, &msg, UNKNOWN_ERROR_MESSAGE.to_string()).await?;

                            // Logging
                            log::error!(
                                "Settings Currency Conversion - Error setting currency conversion for chat {}: {}",
                                chat_id,
                                err.to_string()
                                );
                        }
                    }
                    delete_bot_messages(&bot, &chat_id, messages).await?;
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
