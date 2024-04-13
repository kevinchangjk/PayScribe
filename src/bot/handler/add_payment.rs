use teloxide::{payloads::SendMessageSetters, prelude::*, types::Message};

use crate::bot::{
    dispatcher::State,
    handler::utils::{
        display_balances, display_currency_amount, display_debts, display_username, make_keyboard,
        parse_username, process_debts, HandlerResult, UserDialogue, DEBT_EQUAL_DESCRIPTION_MESSAGE,
        DEBT_EQUAL_INSTRUCTIONS_MESSAGE, DEBT_EXACT_DESCRIPTION_MESSAGE,
        DEBT_EXACT_INSTRUCTIONS_MESSAGE, DEBT_RATIO_DESCRIPTION_MESSAGE,
        DEBT_RATIO_INSTRUCTIONS_MESSAGE, NO_TEXT_MESSAGE, TOTAL_INSTRUCTIONS_MESSAGE,
        UNKNOWN_ERROR_MESSAGE,
    },
    processor::add_payment,
};

use super::{
    utils::{make_keyboard_debt_selection, parse_currency_amount},
    Currency,
};

/* Utilities */
#[derive(Clone, Debug)]
pub struct AddPaymentParams {
    chat_id: String,
    sender_id: String,
    sender_username: String,
    datetime: String,
    description: Option<String>,
    creditor: Option<String>,
    currency: Option<Currency>,
    total: Option<i64>,
    debts: Option<Vec<(String, i64)>>,
}

#[derive(Clone, Debug)]
pub enum AddPaymentEdit {
    Description,
    Creditor,
    Total,
    DebtsEqual,
    DebtsExact,
    DebtsRatio,
}

#[derive(Clone, Debug)]
pub enum AddDebtsFormat {
    Equal,
    Exact,
    Ratio,
}

const CANCEL_MESSAGE: &str =
    "Sure, I've cancelled adding the payment. No changes have been made! 👌";

/* Displays a payment entry (being added) in String format.
*/
fn display_add_payment(payment: &AddPaymentParams) -> String {
    let description = match &payment.description {
        Some(desc) => format!("Description: {}\n", desc),
        None => "".to_string(),
    };
    let creditor = match &payment.creditor {
        Some(cred) => format!("Payer: {}\n", display_username(cred)),
        None => "".to_string(),
    };
    let total = match &payment.total {
        Some(total) => match &payment.currency {
            Some(currency) => format!(
                "Total: {}\n",
                display_currency_amount(*total, currency.clone())
            ),
            None => "".to_string(),
        },
        None => "".to_string(),
    };
    let debts = match &payment.debts {
        Some(debts) => match &payment.currency {
            Some(currency) => format!("Split:\n{}", display_debts(&debts, currency.1)),
            None => "".to_string(),
        },
        None => "".to_string(),
    };

    format!("{}{}{}{}\n", description, creditor, total, debts)
}

/* Add a payment entry in a group chat.
 * Displays an overview of the current details provided.
 * Is not a normal endpoint function, just a temporary transition function.
 */
async fn display_add_overview(
    bot: Bot,
    dialogue: UserDialogue,
    payment: AddPaymentParams,
) -> HandlerResult {
    let buttons = vec!["Cancel", "Edit", "Confirm"];
    let keyboard = make_keyboard(buttons, Some(3));

    bot.send_message(payment.chat_id.clone(), format!("Here's what I've gathered!\n\n{}Do you want to confirm this entry? Or do you want to edit anything?", display_add_payment(&payment)))
        .reply_markup(keyboard)
        .await?;
    dialogue.update(State::AddConfirm { payment }).await?;
    Ok(())
}

/* Add a payment entry in a group chat.
 * Displays a button menu for user to choose which part of the payment details to edit.
 */
async fn display_add_edit_menu(
    bot: Bot,
    dialogue: UserDialogue,
    payment: AddPaymentParams,
    query: CallbackQuery,
) -> HandlerResult {
    let buttons = vec!["Description", "Payer", "Total", "Splits", "Back"];
    let keyboard = make_keyboard(buttons, Some(2));

    if let Some(Message { id, chat, .. }) = query.message {
        bot.edit_message_text(
            chat.id,
            id,
            format!(
                "{}Sure! What would you like to edit?",
                display_add_payment(&payment)
            ),
        )
        .reply_markup(keyboard)
        .await?;
        dialogue.update(State::AddEditMenu { payment }).await?;
    }
    Ok(())
}

/* Parses a string representing debts, and handles it accordingly
*/
async fn handle_debts(
    bot: Bot,
    dialogue: UserDialogue,
    msg: Message,
    payment: AddPaymentParams,
    debts_format: AddDebtsFormat,
) -> HandlerResult {
    let error_msg = match debts_format {
        AddDebtsFormat::Equal => DEBT_EQUAL_INSTRUCTIONS_MESSAGE,
        AddDebtsFormat::Exact => DEBT_EXACT_INSTRUCTIONS_MESSAGE,
        AddDebtsFormat::Ratio => DEBT_RATIO_INSTRUCTIONS_MESSAGE,
    };
    match msg.text() {
        Some(text) => {
            let debts = process_debts(
                debts_format,
                text,
                &payment.creditor,
                payment.currency.clone(),
                payment.total,
            );
            if let Err(err) = debts {
                bot.send_message(msg.chat.id, format!("{}\n\n{error_msg}", err.to_string()))
                    .await?;
                return Ok(());
            }

            let new_payment = AddPaymentParams {
                chat_id: payment.chat_id,
                sender_id: payment.sender_id,
                sender_username: payment.sender_username,
                datetime: payment.datetime,
                description: payment.description,
                creditor: payment.creditor,
                currency: payment.currency,
                total: payment.total,
                debts: Some(debts?),
            };

            display_add_overview(bot, dialogue, new_payment).await?;
        }
        None => {
            bot.send_message(msg.chat.id, error_msg).await?;
        }
    }
    Ok(())
}

/* Calls processor to execute the adding of the payment entry.
*/
async fn call_processor_add_payment(
    bot: Bot,
    dialogue: UserDialogue,
    payment: AddPaymentParams,
    query: CallbackQuery,
) -> HandlerResult {
    if let Some(Message { id, chat, .. }) = query.message {
        let payment_clone = payment.clone();
        let description = match payment.description {
            Some(desc) => desc,
            None => {
                log::error!(
                    "Add Payment Submission - Description not found for payment: {:?}",
                    payment_clone
                );
                bot.edit_message_text(chat.id, id, UNKNOWN_ERROR_MESSAGE)
                    .await?;
                dialogue.exit().await?;
                return Ok(());
            }
        };
        let creditor = match payment.creditor {
            Some(cred) => cred,
            None => {
                log::error!(
                    "Add Payment Submission - Creditor not found for payment: {:?}",
                    payment_clone
                );
                bot.edit_message_text(chat.id, id, UNKNOWN_ERROR_MESSAGE)
                    .await?;
                dialogue.exit().await?;
                return Ok(());
            }
        };
        let currency = match payment.currency {
            Some(curr) => curr,
            None => {
                log::error!(
                    "Add Payment Submission - Currency not found for payment: {:?}",
                    payment_clone
                );
                bot.edit_message_text(chat.id, id, UNKNOWN_ERROR_MESSAGE)
                    .await?;
                dialogue.exit().await?;
                return Ok(());
            }
        };
        let total = match payment.total {
            Some(tot) => tot,
            None => {
                log::error!(
                    "Add Payment Submission - Total not found for payment: {:?}",
                    payment_clone
                );
                bot.edit_message_text(chat.id, id, UNKNOWN_ERROR_MESSAGE)
                    .await?;
                dialogue.exit().await?;
                return Ok(());
            }
        };
        let debts = match payment.debts {
            Some(debts) => debts,
            None => {
                log::error!(
                    "Add Payment Submission - Debts not found for payment: {:?}",
                    payment_clone
                );
                bot.edit_message_text(chat.id, id, UNKNOWN_ERROR_MESSAGE)
                    .await?;
                dialogue.exit().await?;
                return Ok(());
            }
        };
        let payment_overview = display_add_payment(&payment_clone);
        let updated_balances = add_payment(
            payment.chat_id,
            payment.sender_username,
            payment.sender_id,
            payment.datetime,
            &description,
            &creditor,
            &currency.0,
            total,
            debts,
        );
        match updated_balances {
            Err(err) => {
                log::error!(
                    "Add Payment Submission - Processor failed to update balances for user {} in chat {} with payment {:?}: {}",
                    payment_clone.sender_id,
                    payment_clone.chat_id,
                    payment_clone,
                    err.to_string()
                    );
                bot.edit_message_text(
                    chat.id,
                    id,
                    format!(
                        "❓ Hmm, something went wrong! Sorry, I can't add the payment right now."
                    ),
                )
                .await?;
            }
            Ok(balances) => {
                log::info!(
                    "Add Payment Submission - Processor updated balances successfully for user {} in chat {}: {:?}",
                    payment_clone.sender_id,
                    payment_clone.chat_id,
                    payment_clone
                    );
                bot.edit_message_text(
                    chat.id,
                    id,
                    format!(
                        "🎉 I've added the payment! 🎉\n\n{}Here are the updated balances:\n{}",
                        payment_overview,
                        display_balances(&balances)
                    ),
                )
                .await?;
            }
        }
        dialogue.exit().await?;
    }
    Ok(())
}

/* Action handler functions */

/* Handles a repeated call to add payment entry.
 * Does nothing, simply notifies the user.
 */
pub async fn handle_repeated_add_payment(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(
        msg.chat.id,
        "🚫 You are already adding a payment entry! Please complete or cancel the current operation before starting a new one.",
        ).await?;
    Ok(())
}

/* Cancels the add payment operation.
 * Can be called at any step of the process.
 */
pub async fn cancel_add_payment(bot: Bot, dialogue: UserDialogue, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, CANCEL_MESSAGE).await?;
    dialogue.exit().await?;
    Ok(())
}

/* Blocks user command.
 * Called when user attempts to start another operation in the middle of adding a payment.
 */
pub async fn block_add_payment(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(
        msg.chat.id,
        "🚫 You are currently adding a payment entry! Please complete or cancel the current payment entry before starting another command.",
        ).await?;
    Ok(())
}

/* Add a payment entry in a group chat.
 * Bot will ask for user to send messages to fill in required information,
 * before presenting the compiled information for confirmation with a menu.
 */
pub async fn action_add_payment(bot: Bot, dialogue: UserDialogue, msg: Message) -> HandlerResult {
    bot.send_message(
        msg.chat.id,
        format!("Alright!\nWhat's the description for this new payment?"),
    )
    .await?;
    dialogue.update(State::AddDescription).await?;
    Ok(())
}

/* Add a payment entry in a group chat.
 * Bot receives a description string from user, and proceeds to ask for creditor.
 */
pub async fn action_add_description(
    bot: Bot,
    dialogue: UserDialogue,
    msg: Message,
) -> HandlerResult {
    match msg.text() {
        Some(text) => {
            let user = msg.from();
            if let Some(user) = user {
                if let Some(username) = &user.username {
                    let username = parse_username(username);

                    if let Err(err) = username {
                        log::error!(
                            "Add Payment Description - Failed to parse username for user {}: {}",
                            user.id,
                            err.to_string()
                        );
                        return Ok(());
                    }

                    let payment = AddPaymentParams {
                        chat_id: msg.chat.id.to_string(),
                        sender_id: user.id.to_string(),
                        sender_username: username?,
                        datetime: msg.date.to_string(),
                        description: Some(text.to_string()),
                        creditor: None,
                        currency: None,
                        total: None,
                        debts: None,
                    };
                    bot.send_message(
                        msg.chat.id,
                        format!(
                            "{}Great! What's the Telegram username of the one who paid?",
                            display_add_payment(&payment)
                        ),
                    )
                    .await?;
                    dialogue.update(State::AddCreditor { payment }).await?;
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

/* Add a payment entry in a group chat.
 * Bot receives a creditor string from user, and proceeds to ask for total.
 */
pub async fn action_add_creditor(
    bot: Bot,
    dialogue: UserDialogue,
    msg: Message,
    payment: AddPaymentParams,
) -> HandlerResult {
    match msg.text() {
        Some(text) => {
            let text = parse_username(text);

            if let Err(err) = text {
                bot.send_message(msg.chat.id, format!("{}", err.to_string()))
                    .await?;
                return Ok(());
            }

            let new_payment = AddPaymentParams {
                chat_id: payment.chat_id,
                sender_id: payment.sender_id,
                sender_username: payment.sender_username,
                datetime: payment.datetime,
                description: payment.description,
                creditor: Some(text?),
                currency: None,
                total: None,
                debts: None,
            };
            bot.send_message(
                msg.chat.id,
                format!(
                    "{}Nice! How much was the total amount?\n\nOptional: You may also enter the currency of the amount. {TOTAL_INSTRUCTIONS_MESSAGE}",
                    display_add_payment(&new_payment)
                    ),
                    )
                .await?;
            dialogue
                .update(State::AddTotal {
                    payment: new_payment,
                })
                .await?;
        }
        None => {
            bot.send_message(msg.chat.id, format!("{NO_TEXT_MESSAGE}"))
                .await?;
        }
    }
    Ok(())
}

/* Add a payment entry in a group chat.
 * Bot receives a total f64 from user, and proceeds to ask for debts.
 */
pub async fn action_add_total(
    bot: Bot,
    dialogue: UserDialogue,
    msg: Message,
    payment: AddPaymentParams,
) -> HandlerResult {
    match msg.text() {
        Some(text) => {
            let currency_amount = parse_currency_amount(text);
            match currency_amount {
                Ok((total, currency)) => {
                    let new_payment = AddPaymentParams {
                        chat_id: payment.chat_id,
                        sender_id: payment.sender_id,
                        sender_username: payment.sender_username,
                        datetime: payment.datetime,
                        description: payment.description,
                        creditor: payment.creditor,
                        currency: Some(currency),
                        total: Some(total),
                        debts: None,
                    };
                    bot.send_message(
                        msg.chat.id,
                        format!(
                            "{}Cool! How are we splitting this?\n\n{DEBT_EQUAL_DESCRIPTION_MESSAGE}{DEBT_EXACT_DESCRIPTION_MESSAGE}{DEBT_RATIO_DESCRIPTION_MESSAGE}",
                            display_add_payment(&new_payment)
                            ),
                            )
                        .reply_markup(make_keyboard_debt_selection())
                        .await?;
                    dialogue
                        .update(State::AddDebtSelection {
                            payment: new_payment,
                        })
                        .await?;
                }
                Err(err) => {
                    bot.send_message(
                        msg.chat.id,
                        format!("{}\n\n{TOTAL_INSTRUCTIONS_MESSAGE}", err.to_string()),
                    )
                    .await?;
                    return Ok(());
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

/* Add a payment entry in a group chat.
 * Bot receives a callback query from the user indicating how they want to split.
 * No Cancel button required.
 */
pub async fn action_add_debt_selection(
    bot: Bot,
    dialogue: UserDialogue,
    query: CallbackQuery,
    payment: AddPaymentParams,
) -> HandlerResult {
    if let Some(button) = &query.data {
        bot.answer_callback_query(query.id.to_string()).await?;

        match button.as_str() {
            "Equal" => {
                if let Some(Message { id, chat, .. }) = query.message {
                    bot.edit_message_text(
                        chat.id,
                        id,
                        format!(
                            "{}Okay, who is involved in the payment?\n\n{DEBT_EQUAL_INSTRUCTIONS_MESSAGE}",
                            display_add_payment(&payment)
                            ),
                            )
                        .await?;
                    dialogue
                        .update(State::AddDebt {
                            payment,
                            debts_format: AddDebtsFormat::Equal,
                        })
                        .await?;
                }
            }
            "Exact" => {
                if let Some(Message { id, chat, .. }) = query.message {
                    bot.edit_message_text(
                        chat.id,
                        id,
                        format!(
                            "{}Okay, who is involved and how much do they owe?\n\n{DEBT_EXACT_INSTRUCTIONS_MESSAGE}",
                            display_add_payment(&payment))
                        ).await?;
                    dialogue
                        .update(State::AddDebt {
                            payment,
                            debts_format: AddDebtsFormat::Exact,
                        })
                        .await?;
                }
            }
            "Ratio" => {
                if let Some(Message { id, chat, .. }) = query.message {
                    bot.edit_message_text(
                        chat.id,
                        id,
                        format!(
                            "{}Okay, who is involved and what proportions do they owe?\n\n{DEBT_RATIO_INSTRUCTIONS_MESSAGE}",
                            display_add_payment(&payment))
                        ).await?;
                    dialogue
                        .update(State::AddDebt {
                            payment,
                            debts_format: AddDebtsFormat::Ratio,
                        })
                        .await?;
                }
            }
            _ => {
                log::error!("Add Payment Debt Selection - Invalid button for user {} in chat {} with payment {:?}: {}",
                            payment.sender_id, payment.chat_id, payment, button);
            }
        }
    }
    Ok(())
}

/* Add a payment entry in a group chat.
 * Bot receives a Debt from user, and checks if the total amounts tally.
 * If so, it presents an overview. Else, it asks for more debts.
 */
pub async fn action_add_debt(
    bot: Bot,
    dialogue: UserDialogue,
    msg: Message,
    (payment, debts_format): (AddPaymentParams, AddDebtsFormat),
) -> HandlerResult {
    handle_debts(bot, dialogue, msg, payment, debts_format).await
}

/* Add a payment entry in a group chat.
 * Bot receives a callback query from a button menu, on user decision after seeing the overview.
 * If user chooses to edit, proceed to edit.
 * If user confirms, proceeds to add the payment.
 */
pub async fn action_add_confirm(
    bot: Bot,
    dialogue: UserDialogue,
    payment: AddPaymentParams,
    query: CallbackQuery,
) -> HandlerResult {
    if let Some(button) = &query.data {
        bot.answer_callback_query(query.id.to_string()).await?;

        match button.as_str() {
            "Cancel" => {
                if let Some(Message { id, chat, .. }) = query.message {
                    bot.edit_message_text(chat.id, id, CANCEL_MESSAGE).await?;
                    dialogue.exit().await?;
                }
            }
            "Edit" => {
                display_add_edit_menu(bot, dialogue, payment, query).await?;
            }
            "Confirm" => {
                call_processor_add_payment(bot, dialogue, payment, query).await?;
            }
            _ => {
                log::error!("Add Payment Confirm - Invalid button for user {} in chat {} with payment {:?}: {}",
                            payment.sender_id, payment.chat_id, payment, button);
            }
        }
    }
    Ok(())
}

/* Add a payment entry in a group chat.
 * Bot receives a callback query on user decision on what to edit.
 * If the user chooses to go back, return to confirm page.
 */
pub async fn action_add_edit_menu(
    bot: Bot,
    dialogue: UserDialogue,
    payment: AddPaymentParams,
    query: CallbackQuery,
) -> HandlerResult {
    if let Some(button) = &query.data {
        bot.answer_callback_query(query.id.to_string()).await?;

        if let Some(Message { id, chat, .. }) = query.message {
            let payment_clone = payment.clone();
            match button.as_str() {
                "Description" => {
                    bot.edit_message_text(
                        chat.id,
                        id,
                        format!(
                            "Current description: {}\n\nWhat do you want the new description to be?",
                            payment_clone.description.unwrap()
                            ),
                            )
                        .await?;
                    dialogue
                        .update(State::AddEdit {
                            payment,
                            edit: AddPaymentEdit::Description,
                        })
                        .await?;
                }
                "Payer" => {
                    bot.edit_message_text(
                        chat.id,
                        id,
                        format!(
                            "Current payer: {}\n\nWho should the payer be?",
                            display_username(&payment_clone.creditor.unwrap())
                        ),
                    )
                    .await?;
                    dialogue
                        .update(State::AddEdit {
                            payment,
                            edit: AddPaymentEdit::Creditor,
                        })
                        .await?;
                }
                "Total" => {
                    bot.edit_message_text(
                        chat.id,
                        id,
                        format!(
                            "Current total: {}\n\nWhat should the total be?\n\nOptional: You may also enter the currency of the amount.{TOTAL_INSTRUCTIONS_MESSAGE}",
                            payment_clone.total.unwrap()
                        ),
                    )
                    .await?;
                    dialogue
                        .update(State::AddEdit {
                            payment,
                            edit: AddPaymentEdit::Total,
                        })
                        .await?;
                }
                "Splits" => {
                    bot.edit_message_text(
                        chat.id,
                        id,
                        format!(
                            "Current splits:\n{}\nHow are we splitting this?\n\n{DEBT_EQUAL_DESCRIPTION_MESSAGE}{DEBT_EXACT_DESCRIPTION_MESSAGE}{DEBT_RATIO_DESCRIPTION_MESSAGE}",
                            display_debts(&payment_clone.debts.unwrap(), payment_clone.currency.unwrap().1)
                            ),
                            ).reply_markup(make_keyboard_debt_selection())
                        .await?;
                    dialogue.update(State::AddDebtSelection { payment }).await?;
                }
                "Back" => {
                    display_add_overview(bot, dialogue, payment).await?;
                }
                _ => {
                    log::error!("Add Payment Edit Menu - Invalid button for user {} in chat {} with payment {:?}: {}",
                                payment_clone.sender_id, payment_clone.chat_id, payment_clone, button);
                }
            }
        }
    }

    Ok(())
}

/* Add a payment entry in a group chat.
 * Bot receives a callback query on user decision on what to edit.
 * If the user chooses to go back, return to confirm page.
 */
pub async fn action_add_edit(
    bot: Bot,
    dialogue: UserDialogue,
    msg: Message,
    (payment, edit): (AddPaymentParams, AddPaymentEdit),
) -> HandlerResult {
    match msg.text() {
        Some(text) => match edit {
            AddPaymentEdit::Description => {
                let new_payment = AddPaymentParams {
                    chat_id: payment.chat_id,
                    sender_id: payment.sender_id,
                    sender_username: payment.sender_username,
                    datetime: payment.datetime,
                    description: Some(text.to_string()),
                    creditor: payment.creditor,
                    currency: payment.currency,
                    total: payment.total,
                    debts: payment.debts,
                };
                display_add_overview(bot, dialogue, new_payment).await?;
            }
            AddPaymentEdit::Creditor => {
                let username = parse_username(text);

                if let Err(err) = username {
                    bot.send_message(msg.chat.id, err.to_string()).await?;
                    return Ok(());
                }

                let new_payment = AddPaymentParams {
                    chat_id: payment.chat_id,
                    sender_id: payment.sender_id,
                    sender_username: payment.sender_username,
                    datetime: payment.datetime,
                    description: payment.description,
                    creditor: Some(username?),
                    currency: payment.currency,
                    total: payment.total,
                    debts: payment.debts,
                };
                display_add_overview(bot, dialogue, new_payment).await?;
            }
            AddPaymentEdit::Total => {
                let currency_amount = parse_currency_amount(text);
                match currency_amount {
                    Ok((total, currency)) => {
                        let new_payment = AddPaymentParams {
                            chat_id: payment.chat_id,
                            sender_id: payment.sender_id,
                            sender_username: payment.sender_username,
                            datetime: payment.datetime,
                            description: payment.description,
                            creditor: payment.creditor,
                            currency: Some(currency),
                            total: Some(total),
                            debts: payment.debts,
                        };
                        bot.send_message(
                    msg.chat.id,
                    format!("How are we splitting this?\n\n{DEBT_EQUAL_DESCRIPTION_MESSAGE}{DEBT_EXACT_DESCRIPTION_MESSAGE}{DEBT_RATIO_DESCRIPTION_MESSAGE}",),
                    ).reply_markup(make_keyboard_debt_selection())
                    .await?;
                        dialogue
                            .update(State::AddDebtSelection {
                                payment: new_payment,
                            })
                            .await?;
                    }
                    Err(err) => {
                        bot.send_message(msg.chat.id, err.to_string()).await?;
                        return Ok(());
                    }
                }
            }
            AddPaymentEdit::DebtsEqual => {
                handle_debts(bot, dialogue, msg, payment, AddDebtsFormat::Equal).await?;
            }
            AddPaymentEdit::DebtsExact => {
                handle_debts(bot, dialogue, msg, payment, AddDebtsFormat::Exact).await?;
            }
            AddPaymentEdit::DebtsRatio => {
                handle_debts(bot, dialogue, msg, payment, AddDebtsFormat::Ratio).await?;
            }
        },
        None => {
            bot.send_message(msg.chat.id, format!("{NO_TEXT_MESSAGE}"))
                .await?;
        }
    }

    Ok(())
}
