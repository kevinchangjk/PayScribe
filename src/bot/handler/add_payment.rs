use teloxide::{payloads::SendMessageSetters, prelude::*, types::Message};

use crate::bot::{
    dispatcher::{HandlerResult, State, UserDialogue},
    handler::{
        general::{NO_TEXT_MESSAGE, UNKNOWN_ERROR_MESSAGE},
        utils::{
            display_balances, display_debts, make_keyboard, parse_amount, parse_username,
            process_debts,
        },
    },
    processor::add_payment,
    BotError,
};

/* Utilities */
const HEADER_MESSAGE: &str = "Adding a new payment entry!\n\n";
const FOOTER_MESSAGE: &str = "\n\n";
const DEBT_INSTRUCTIONS_MESSAGE: &str =
    "Enter the usernames and the amounts as follows: \n\n@user1 amount1, @user2 amount2, etc.\n\n";

#[derive(Clone, Debug)]
pub struct AddPaymentParams {
    chat_id: String,
    sender_id: String,
    sender_username: String,
    datetime: String,
    description: Option<String>,
    creditor: Option<String>,
    total: Option<f64>,
    debts: Option<Vec<(String, f64)>>,
}

#[derive(Clone, Debug)]
pub enum AddPaymentEdit {
    Description,
    Creditor,
    Total,
    Debts,
}

/* Displays a payment entry (being added) in String format.
 */
fn display_add_payment(payment: &AddPaymentParams) -> String {
    let description = match &payment.description {
        Some(desc) => format!("Description: {}\n", desc),
        None => "".to_string(),
    };
    let creditor = match &payment.creditor {
        Some(cred) => format!("Creditor: {}\n", cred),
        None => "".to_string(),
    };
    let total = match &payment.total {
        Some(total) => format!("Total: {}\n", total),
        None => "".to_string(),
    };
    let debts = match &payment.debts {
        Some(debts) => format!("Split amounts:\n{}", display_debts(&debts)),
        None => "".to_string(),
    };

    format!(
        "Payment Overview:\n{}{}{}{}\n",
        description, creditor, total, debts
    )
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

    bot.send_message(payment.chat_id.clone(), display_add_payment(&payment))
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
    let buttons = vec!["Description", "Creditor", "Total", "Debts", "Back"];
    let keyboard = make_keyboard(buttons, Some(2));

    if let Some(Message { id, chat, .. }) = query.message {
        bot.edit_message_text(
            chat.id,
            id,
            "Which part of the payment details would you like to edit?",
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
    error_msg: String,
) -> HandlerResult {
    match msg.text() {
        Some(text) => {
            let debts = process_debts(text, &payment.creditor, payment.total);
            if let Err(err) = debts {
                log::error!(
                    "Add Payment - Debt parsing failed for user {} in chat {}: {}",
                    payment.sender_id,
                    payment.chat_id,
                    err.to_string()
                );
                bot.send_message(msg.chat.id, format!("{}\n\nWho are we splitting this with?\n{DEBT_INSTRUCTIONS_MESSAGE}{FOOTER_MESSAGE}", err.to_string())).await?;
                return Ok(());
            }

            let new_payment = AddPaymentParams {
                chat_id: payment.chat_id,
                sender_id: payment.sender_id,
                sender_username: payment.sender_username,
                datetime: payment.datetime,
                description: payment.description,
                creditor: payment.creditor,
                total: payment.total,
                debts: Some(debts?),
            };

            log::info!(
                "Add Payment - Debt updated successfully by user {} in chat {}: {:?}",
                new_payment.sender_id,
                new_payment.chat_id,
                new_payment
            );
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
                    format!("{}\nPayment entry failed!", err.to_string()),
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
                        "Payment entry added!\n\n{}Current balances:\n{}",
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
        "You are already adding a payment entry! Please complete or cancel the current operation before starting a new one.",
    ).await?;
    Ok(())
}

/* Cancels the add payment operation.
 * Can be called at any step of the process.
 */
pub async fn cancel_add_payment(bot: Bot, dialogue: UserDialogue, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, "Payment entry cancelled!")
        .await?;
    dialogue.exit().await?;
    Ok(())
}

/* Blocks user command.
 * Called when user attempts to start another operation in the middle of adding a payment.
 */
pub async fn block_add_payment(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(
        msg.chat.id,
        "You are currently adding a payment entry! Please complete or cancel the current payment entry before starting another command.",
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
        format!("{HEADER_MESSAGE}Enter a description for the payment: {FOOTER_MESSAGE}"),
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
                let username = match &user.username {
                    Some(user) => parse_username(user),
                    None => {
                        return Err(BotError::UserError(format!(
                            "Add Payment - Username not found for user ID {}",
                            user.id.to_string()
                        )));
                    }
                };
                let payment = AddPaymentParams {
                    chat_id: msg.chat.id.to_string(),
                    sender_id: user.id.to_string(),
                    sender_username: username,
                    datetime: msg.date.to_string(),
                    description: Some(text.to_string()),
                    creditor: None,
                    total: None,
                    debts: None,
                };
                log::info!(
                    "Add Payment - Description updated successfully for user {} in chat {}: {:?}",
                    payment.sender_id,
                    payment.chat_id,
                    payment
                );
                bot.send_message(
                    msg.chat.id,
                    format!(
                        "{}Enter the username of the one who paid the total: {FOOTER_MESSAGE}",
                        display_add_payment(&payment)
                    ),
                )
                .await?;
                dialogue.update(State::AddCreditor { payment }).await?;
            }
        }
        None => {
            bot.send_message(
                msg.chat.id,
                format!(
                    "{NO_TEXT_MESSAGE}What is the description for the payment?{FOOTER_MESSAGE}"
                ),
            )
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
            let text: String = parse_username(text);
            let new_payment = AddPaymentParams {
                chat_id: payment.chat_id,
                sender_id: payment.sender_id,
                sender_username: payment.sender_username,
                datetime: payment.datetime,
                description: payment.description,
                creditor: Some(text),
                total: None,
                debts: None,
            };
            log::info!(
                "Add Payment - Creditor updated successfully for user {} in chat {}: {:?}",
                new_payment.sender_id,
                new_payment.chat_id,
                new_payment
            );
            bot.send_message(
                msg.chat.id,
                format!(
                    "{}Enter the total amount paid (without currency):{FOOTER_MESSAGE}",
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
            bot.send_message(
                msg.chat.id,
                format!("{NO_TEXT_MESSAGE}What is the username of the payer?{FOOTER_MESSAGE}"),
            )
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
            let total = parse_amount(text);
            if let Err(err) = total {
                bot.send_message(msg.chat.id, format!("{}\n\nWhat is the total amount paid? Please enter the number without any symbols.{FOOTER_MESSAGE}", err.to_string())).await?;
                return Ok(());
            }
            let new_payment = AddPaymentParams {
                chat_id: payment.chat_id,
                sender_id: payment.sender_id,
                sender_username: payment.sender_username,
                datetime: payment.datetime,
                description: payment.description,
                creditor: payment.creditor,
                total: Some(total?),
                debts: None,
            };
            log::info!(
                "Add Payment - Total updated successfully for user {} in chat {}: {:?}",
                new_payment.sender_id,
                new_payment.chat_id,
                new_payment.total
            );
            bot.send_message(
                msg.chat.id,
                format!("{}Who are we splitting this with?\n{DEBT_INSTRUCTIONS_MESSAGE}{FOOTER_MESSAGE}", display_add_payment(&new_payment)),
            )
            .await?;
            dialogue
                .update(State::AddDebt {
                    payment: new_payment,
                })
                .await?;
        }
        None => {
            bot.send_message(
                msg.chat.id,
                format!("{NO_TEXT_MESSAGE}What is the total amount paid?{FOOTER_MESSAGE}"),
            )
            .await?;
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
    payment: AddPaymentParams,
) -> HandlerResult {
    handle_debts(
        bot,
        dialogue,
        msg,
        payment,
        format!("{NO_TEXT_MESSAGE}{DEBT_INSTRUCTIONS_MESSAGE}{FOOTER_MESSAGE}"),
    )
    .await
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
        bot.answer_callback_query(format!("{}", query.id)).await?;

        match button.as_str() {
            "Cancel" => {
                if let Some(Message { id, chat, .. }) = query.message {
                    bot.edit_message_text(chat.id, id, "Payment entry cancelled!")
                        .await?;
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
        bot.answer_callback_query(format!("{}", query.id)).await?;

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
                "Creditor" => {
                    bot.edit_message_text(
                        chat.id,
                        id,
                        format!(
                            "Current creditor: {}\n\nWho should the creditor be?",
                            payment_clone.creditor.unwrap()
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
                            "Current total: {}\n\nWhat should the total be?",
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
                "Debts" => {
                    bot.edit_message_text(
                        chat.id,
                        id,
                        format!(
                            "Current debts:\n{}\n{DEBT_INSTRUCTIONS_MESSAGE}",
                            display_debts(&payment_clone.debts.unwrap())
                        ),
                    )
                    .await?;
                    dialogue
                        .update(State::AddEdit {
                            payment,
                            edit: AddPaymentEdit::Debts,
                        })
                        .await?;
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
                    total: payment.total,
                    debts: payment.debts,
                };
                log::info!(
                    "Add Payment - Description updated successfully for user {} in chat {}: {:?}",
                    new_payment.sender_id,
                    new_payment.chat_id,
                    new_payment
                );
                display_add_overview(bot, dialogue, new_payment).await?;
            }
            AddPaymentEdit::Creditor => {
                let new_payment = AddPaymentParams {
                    chat_id: payment.chat_id,
                    sender_id: payment.sender_id,
                    sender_username: payment.sender_username,
                    datetime: payment.datetime,
                    description: payment.description,
                    creditor: Some(parse_username(text)),
                    total: payment.total,
                    debts: payment.debts,
                };
                log::info!(
                    "Add Payment - Creditor updated successfully for user {} in chat {}: {:?}",
                    new_payment.sender_id,
                    new_payment.chat_id,
                    new_payment
                );
                display_add_overview(bot, dialogue, new_payment).await?;
            }
            AddPaymentEdit::Total => {
                let total = parse_amount(text);
                if let Err(err) = total {
                    bot.send_message(
                        msg.chat.id,
                        format!(
                            "{}\n\nWhat should the total be?{FOOTER_MESSAGE}",
                            err.to_string()
                        ),
                    )
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
                    total: Some(total?),
                    debts: payment.debts,
                };

                log::info!(
                    "Add Payment - Total updated successfully for user {} in chat {}: {:?}",
                    new_payment.sender_id,
                    new_payment.chat_id,
                    new_payment
                );
                bot.send_message(
                    msg.chat.id,
                    format!("Who are we splitting this with?\n{DEBT_INSTRUCTIONS_MESSAGE}"),
                )
                .await?;
                dialogue
                    .update(State::AddEdit {
                        payment: new_payment,
                        edit: AddPaymentEdit::Debts,
                    })
                    .await?;
            }
            AddPaymentEdit::Debts => {
                handle_debts(
                    bot,
                    dialogue,
                    msg,
                    payment,
                    format!("{DEBT_INSTRUCTIONS_MESSAGE}",),
                )
                .await?;
            }
        },
        None => {
            bot.send_message(
                msg.chat.id,
                format!("{NO_TEXT_MESSAGE}What is the new value?{FOOTER_MESSAGE}"),
            )
            .await?;
        }
    }

    Ok(())
}
