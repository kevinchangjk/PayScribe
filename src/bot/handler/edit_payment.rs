use teloxide::{payloads::SendMessageSetters, prelude::*, types::Message};

use crate::bot::{
    dispatcher::State,
    handler::{
        utils::{
            display_balances, display_debts, display_payment, make_keyboard, parse_amount,
            parse_serial_num, parse_username, process_debts, BotError, HandlerResult, UserDialogue,
            COMMAND_VIEW_PAYMENTS, DEBT_INSTRUCTIONS_MESSAGE, NO_TEXT_MESSAGE,
        },
        AddPaymentEdit, Payment,
    },
    processor::edit_payment,
};

/* Utilities */
#[derive(Clone, Debug)]
pub struct EditPaymentParams {
    description: Option<String>,
    creditor: Option<String>,
    total: Option<f64>,
    debts: Option<Vec<(String, f64)>>,
}

const CANCEL_MESSAGE: &str = "Sure, I've cancelled editing the payment. No changes have been made!";

/* Displays a payment entry by combining original entry and edited fields.
 */
fn display_edit_payment(payment: Payment, edited_payment: EditPaymentParams) -> String {
    format!(
        "Description: {}\nPayer: {}\nTotal: {:.2}\n{}",
        edited_payment.description.unwrap_or(payment.description),
        edited_payment.creditor.unwrap_or(payment.creditor),
        edited_payment.total.unwrap_or(payment.total),
        display_debts(&edited_payment.debts.unwrap_or(payment.debts.clone()))
    )
}

/* Edit a payment entry in a group chat.
 * Displays an overview of the current details provided.
 */
async fn display_edit_overview(
    bot: Bot,
    dialogue: UserDialogue,
    msg: Message,
    payment: Payment,
    edited_payment: EditPaymentParams,
    payments: Vec<Payment>,
    page: usize,
) -> HandlerResult {
    let options = vec![
        "Description",
        "Payer",
        "Total",
        "Splits",
        "Cancel",
        "Confirm",
    ];
    let keyboard = make_keyboard(options, Some(4));
    bot.send_message(
        msg.chat.id,
        format!(
            "Sure! What do you want to edit for this payment entry?\n\n{}",
            display_edit_payment(payment.clone(), edited_payment.clone())
        ),
    )
    .reply_markup(keyboard)
    .await?;

    dialogue
        .update(State::EditPayment {
            payment,
            edited_payment,
            payments,
            page,
        })
        .await?;
    Ok(())
}

/* Calls processor to execute the edit of the payment entry.
 */
async fn call_processor_edit_payment(
    bot: Bot,
    dialogue: UserDialogue,
    payment: Payment,
    edited_payment: EditPaymentParams,
    payments: Vec<Payment>,
    page: usize,
    query: CallbackQuery,
) -> HandlerResult {
    if let Some(Message { id, chat, .. }) = query.message {
        let edited_clone = edited_payment.clone();
        let edited = edit_payment(
            &chat.id.to_string(),
            &payment.payment_id,
            edited_payment.description.as_deref(),
            edited_payment.creditor.as_deref(),
            edited_payment.total.as_ref(),
            edited_payment.debts,
        );

        match edited {
            Ok(balances) => {
                let edit_overview = display_edit_payment(payment, edited_clone);
                log::info!(
                    "Edit Payment Submission - payment edited for chat {} with payment {}",
                    chat.id,
                    edit_overview
                );

                match balances {
                    Some(balances) => {
                        bot.edit_message_text(
                            chat.id,
                            id,
                            format!(
                                "I've edited the payment!\n\n{}\nHere are the updated balances:\n{}",
                                edit_overview,
                                display_balances(&balances)
                            ),
                        )
                        .await?;
                    }
                    None => {
                        bot.edit_message_text(
                            chat.id,
                            id,
                            format!(
                                "I've edited the payment!\n\n{}\nThere are no changes to the balances.",
                                edit_overview
                            ),
                        )
                        .await?;
                    }
                }
                dialogue
                    .update(State::ViewPayments { payments, page })
                    .await?;
            }
            Err(err) => {
                log::error!(
                    "Edit Payment Submission - Processor failed to edit payment for chat {} with payment {}: {}",
                    chat.id,
                    display_payment(&payment, 1),
                    err.to_string()
                );
                bot.edit_message_text(
                    chat.id,
                    id,
                    format!(
                        "Hmm, something went wrong! Sorry, I can't edit the payment right now."
                    ),
                )
                .await?;
                dialogue
                    .update(State::ViewPayments { payments, page })
                    .await?;
            }
        }
    }

    Ok(())
}

/* Action handler functions */

/* Handles a repeated call to edit payment entry.
 * Does nothing, simply notifies the user.
 */
pub async fn handle_repeated_edit_payment(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(
        msg.chat.id,
        "You are already editing a payment entry! Please complete or cancel the current operation before starting a new one.",
    ).await?;
    Ok(())
}

/* Cancels the edit payment operation.
 * Can be called at any step of the process.
 */
pub async fn cancel_edit_payment(bot: Bot, dialogue: UserDialogue, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, CANCEL_MESSAGE).await?;
    dialogue.exit().await?;
    Ok(())
}

/* Blocks user command.
 * Called when user attempts to start another operation in the middle of adding a payment.
 */
pub async fn block_edit_payment(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(
        msg.chat.id,
        "You are currently editing a payment entry! Please complete or cancel the current payment entry before starting another command.",
    ).await?;
    Ok(())
}

/* Blocks user command.
 * Called when user attempts to edit payment without first viewing anything.
 */
pub async fn no_edit_payment(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(
        msg.chat.id,
        format!("Please view the payment records first with {COMMAND_VIEW_PAYMENTS}!"),
    )
    .await?;
    Ok(())
}

/* Edits a specified payment.
 * Bot will ask user to choose which part to edit, and ask for new values,
 * before confirming the changes and updating the balances.
 */
pub async fn action_edit_payment(
    bot: Bot,
    dialogue: UserDialogue,
    msg: Message,
    (payments, page): (Vec<Payment>, usize),
    serial_num: String,
) -> HandlerResult {
    let user = msg.from();
    if let Some(_user) = user {
        let parsed_serial = parse_serial_num(&serial_num, payments.len());
        match parsed_serial {
            Ok(serial_num) => {
                let payment = payments[serial_num - 1].clone();
                let edited_payment = EditPaymentParams {
                    description: None,
                    creditor: None,
                    total: None,
                    debts: None,
                };

                display_edit_overview(bot, dialogue, msg, payment, edited_payment, payments, page)
                    .await?;
                return Ok(());
            }
            Err(err) => {
                if payments.len() == 1 {
                    bot.send_message(
                        msg.chat.id,
                        format!("{}\nA valid serial number would be 1.", err.to_string()),
                    )
                    .await?;
                } else {
                    bot.send_message(
                        msg.chat.id,
                        format!(
                            "{}\nA valid serial number is a number from 1 to {}.",
                            err.to_string(),
                            payments.len()
                        ),
                    )
                    .await?;
                }
                return Ok(());
            }
        }
    }
    dialogue.exit().await?;
    log::error!(
        "Edit Payment - User not found in message: {}",
        msg.id.to_string()
    );
    Ok(())
}

/* Edits a specified payment.
 * Bot receives a callback query to confirm the changes.
 */
pub async fn action_edit_payment_confirm(
    bot: Bot,
    dialogue: UserDialogue,
    (payment, edited_payment, payments, page): (Payment, EditPaymentParams, Vec<Payment>, usize),
    query: CallbackQuery,
) -> HandlerResult {
    if let Some(button) = &query.data {
        bot.answer_callback_query(query.id.to_string()).await?;

        if let Some(Message { id, chat, .. }) = &query.message {
            match button.as_str() {
                "Cancel" => {
                    bot.edit_message_text(chat.id, *id, format!("{CANCEL_MESSAGE}"))
                        .await?;
                    dialogue
                        .update(State::ViewPayments { payments, page })
                        .await?;
                }
                "Confirm" => {
                    call_processor_edit_payment(
                        bot,
                        dialogue,
                        payment,
                        edited_payment,
                        payments,
                        page,
                        query,
                    )
                    .await?;
                }
                "Description" => {
                    bot.send_message(
                        chat.id,
                        format!("Current description: {}\n\nWhat do you want the new description to be?",
                        payment.description),
                    )
                    .await?;
                    dialogue
                        .update(State::EditPaymentDetails {
                            payment,
                            edited_payment,
                            edit: AddPaymentEdit::Description,
                            payments,
                            page,
                        })
                        .await?;
                }
                "Payer" => {
                    bot.send_message(
                        chat.id,
                        format!(
                            "Current payer: {}\n\nWho should the payer be?",
                            payment.creditor
                        ),
                    )
                    .await?;
                    dialogue
                        .update(State::EditPaymentDetails {
                            payment,
                            edited_payment,
                            edit: AddPaymentEdit::Creditor,
                            payments,
                            page,
                        })
                        .await?;
                }
                "Total" => {
                    bot.send_message(
                        chat.id,
                        format!(
                            "Current total: {}\n\nWhat should the total be?",
                            payment.total
                        ),
                    )
                    .await?;
                    dialogue
                        .update(State::EditPaymentDetails {
                            payment,
                            edited_payment,
                            edit: AddPaymentEdit::Total,
                            payments,
                            page,
                        })
                        .await?;
                }
                "Splits" => {
                    bot.send_message(
                        chat.id,
                        format!(
                            "Current splits: {}\n\nHow are we splitting this?",
                            display_debts(&payment.debts)
                        ),
                    )
                    .await?;
                    dialogue
                        .update(State::EditPaymentDetails {
                            payment,
                            edited_payment,
                            edit: AddPaymentEdit::Debts,
                            payments,
                            page,
                        })
                        .await?;
                }
                _ => {
                    log::error!(
                        "Edit Payment Menu - Invalid button in chat {}: {}",
                        chat.id,
                        button
                    );
                    dialogue
                        .update(State::ViewPayments { payments, page })
                        .await?;
                }
            }
        }
    }

    Ok(())
}

/* Edits a specified payment.
 * Bot receives a text message, and depending on the edit enum, edits the corresponding part.
 */
pub async fn action_edit_payment_edit(
    bot: Bot,
    dialogue: UserDialogue,
    msg: Message,
    (payment, edited_payment, edit, payments, page): (
        Payment,
        EditPaymentParams,
        AddPaymentEdit,
        Vec<Payment>,
        usize,
    ),
) -> HandlerResult {
    match msg.text() {
        Some(text) => match edit {
            AddPaymentEdit::Description => {
                let new_edited_payment = EditPaymentParams {
                    description: Some(text.to_string()),
                    creditor: edited_payment.creditor,
                    total: edited_payment.total,
                    debts: edited_payment.debts,
                };
                log::info!(
                    "Edit Payment - Description updated successfully for user {} in chat {}: {:?}",
                    msg.from().unwrap().id,
                    msg.chat.id,
                    display_edit_payment(payment.clone(), new_edited_payment.clone())
                );
                display_edit_overview(
                    bot,
                    dialogue,
                    msg,
                    payment,
                    new_edited_payment,
                    payments,
                    page,
                )
                .await?;
            }
            AddPaymentEdit::Creditor => {
                let new_edited_payment = EditPaymentParams {
                    description: edited_payment.description,
                    creditor: Some(parse_username(text)),
                    total: edited_payment.total,
                    debts: edited_payment.debts,
                };
                log::info!(
                    "Edit Payment - Creditor updated successfully for user {} in chat {}: {:?}",
                    msg.from().unwrap().id,
                    msg.chat.id,
                    display_edit_payment(payment.clone(), new_edited_payment.clone())
                );
                display_edit_overview(
                    bot,
                    dialogue,
                    msg,
                    payment,
                    new_edited_payment,
                    payments,
                    page,
                )
                .await?;
            }
            AddPaymentEdit::Total => {
                let total = parse_amount(text);
                if let Err(err) = total {
                    bot.send_message(
                        msg.chat.id,
                        format!("{}\n\nWhat should the total be?", err.to_string()),
                    )
                    .await?;
                    return Ok(());
                }

                let new_edited_payment = EditPaymentParams {
                    description: edited_payment.description,
                    creditor: edited_payment.creditor,
                    total: Some(total.unwrap()),
                    debts: edited_payment.debts,
                };

                log::info!(
                    "Edit Payment - Total updated successfully for user {} in chat {}: {:?}",
                    msg.from().unwrap().id,
                    msg.chat.id,
                    display_edit_payment(payment.clone(), new_edited_payment.clone())
                );

                bot.send_message(
                    msg.chat.id,
                    format!("How are we splitting this new total?\n{DEBT_INSTRUCTIONS_MESSAGE}"),
                )
                .await?;
                dialogue
                    .update(State::EditPaymentDetails {
                        payment,
                        edited_payment: new_edited_payment,
                        edit: AddPaymentEdit::Debts,
                        payments,
                        page,
                    })
                    .await?;
            }
            AddPaymentEdit::Debts => match msg.text() {
                Some(text) => {
                    let debts = process_debts(
                        text,
                        &edited_payment
                            .creditor
                            .clone()
                            .or(Some(payment.creditor.clone())),
                        edited_payment.total.or(Some(payment.total)),
                    );
                    if let Err(err) = debts {
                        log::error!(
                            "Edit Payment - Debt parsing failed for user {} in chat {}: {}",
                            msg.from().unwrap().id,
                            msg.chat.id,
                            err.to_string()
                        );
                        bot.send_message(
                            msg.chat.id,
                            format!("{}\n\n{DEBT_INSTRUCTIONS_MESSAGE}", err.to_string()),
                        )
                        .await?;
                        return Ok(());
                    }

                    let new_edited_payment = EditPaymentParams {
                        description: edited_payment.description,
                        creditor: edited_payment.creditor,
                        total: edited_payment.total,
                        debts: Some(debts.unwrap()),
                    };

                    log::info!(
                        "Edit Payment - Creditor updated successfully for user {} in chat {}: {:?}",
                        msg.from().unwrap().id,
                        msg.chat.id,
                        display_edit_payment(payment.clone(), new_edited_payment.clone())
                    );
                    display_edit_overview(
                        bot,
                        dialogue,
                        msg,
                        payment,
                        new_edited_payment,
                        payments,
                        page,
                    )
                    .await?;
                }
                None => {
                    bot.send_message(msg.chat.id, format!("{DEBT_INSTRUCTIONS_MESSAGE}"))
                        .await?;
                }
            },
        },
        None => {
            bot.send_message(msg.chat.id, format!("{NO_TEXT_MESSAGE}"))
                .await?;
        }
    }

    Ok(())
}
