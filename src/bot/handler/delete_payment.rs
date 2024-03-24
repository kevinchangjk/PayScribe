use teloxide::{payloads::SendMessageSetters, prelude::*, types::Message};

use crate::bot::{
    dispatcher::{HandlerResult, State, UserDialogue},
    handler::utils::{display_balances, make_keyboard},
    processor::delete_payment,
    BotError,
};

use super::{
    utils::{display_payment, parse_serial_num},
    Payment,
};

/* Utilities */
const HEADER_MESSAGE: &str = "Adding a new payment entry!\n\n";
const FOOTER_MESSAGE: &str = "\n\n";

/* Action handler functions */

/* Handles a repeated call to delete payment entry.
 * Does nothing, simply notifies the user.
 */
pub async fn handle_repeated_delete_payment(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(
        msg.chat.id,
        "You are already deleting a payment entry! Please complete or cancel the current operation before starting a new one.",
    ).await?;
    Ok(())
}

/* Cancels the edit payment operation.
 * Can be called at any step of the process.
 */
pub async fn cancel_delete_payment(
    bot: Bot,
    dialogue: UserDialogue,
    msg: Message,
) -> HandlerResult {
    bot.send_message(msg.chat.id, "Payment deletion cancelled, no changes made!")
        .await?;
    dialogue.exit().await?;
    Ok(())
}

/* Blocks user command.
 * Called when user attempts to start another operation in the middle of adding a payment.
 */
pub async fn block_delete_payment(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(
        msg.chat.id,
        "You are currently deleting a payment entry! Please complete or cancel the current payment entry before starting another command.",
    ).await?;
    Ok(())
}

/* Blocks user command.
 * Called when user attempts to delete payment without first viewing anything.
 */
pub async fn no_delete_payment(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(
        msg.chat.id,
        "Please view the payment records first with /viewpayments!",
    )
    .await?;
    Ok(())
}

/* Deletes a specified payment.
 * Bot will ask user for confirmation (or cancellation),
 * before confirming the changes and updating the balances.
 */
pub async fn action_delete_payment(
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
                let keyboard = make_keyboard(vec!["Cancel", "Confirm"], Some(2));
                bot.send_message(
                    msg.chat.id,
                    format!(
                        "Are you sure you want to delete the following payment?\n\n{}",
                        display_payment(&payment, serial_num)
                    ),
                )
                .reply_markup(keyboard)
                .await?;
                dialogue
                    .update(State::DeletePayment {
                        payment,
                        payments,
                        page,
                    })
                    .await?;
                return Ok(());
            }
            Err(err) => {
                bot.send_message(
                    msg.chat.id,
                    format!(
                        "{}\nPlease choose a number between 1 and {}.",
                        err.to_string(),
                        payments.len()
                    ),
                )
                .await?;
                return Ok(());
            }
        }
    }
    dialogue.exit().await?;
    Err(BotError::UserError(
        "Unable to delete payment: User not found".to_string(),
    ))
}

/* Deletes a specified payment.
 * Bot receives a callback query from the user, and will either confirm or cancel the deletion.
 */
pub async fn action_delete_payment_confirm(
    bot: Bot,
    dialogue: UserDialogue,
    (payment, payments, page): (Payment, Vec<Payment>, usize),
    query: CallbackQuery,
) -> HandlerResult {
    if let Some(button) = &query.data {
        bot.answer_callback_query(format!("{}", query.id)).await?;

        if let Some(Message { id, chat, .. }) = query.message {
            match button.as_str() {
                "Cancel" => {
                    bot.edit_message_text(
                        chat.id,
                        id,
                        format!("Payment deletion cancelled, no changes made!"),
                    )
                    .await?;
                    dialogue
                        .update(State::ViewPayments { payments, page })
                        .await?;
                }
                "Confirm" => {
                    let payment_id = &payment.payment_id;
                    let deletion = delete_payment(&chat.id.to_string(), payment_id);

                    match deletion {
                        Ok(balances) => {
                            log::info!(
                                "Delete Payment Submission - payment deleted for chat {} with payment {}",
                                chat.id,
                                display_payment(&payment, 1)
                            );
                            bot.edit_message_text(
                                chat.id,
                                id,
                                format!(
                                    "Payment successfully deleted!\n\nCurrent balances:\n{}",
                                    display_balances(&balances)
                                ),
                            )
                            .await?;
                            dialogue
                                .update(State::ViewPayments { payments, page })
                                .await?;
                        }
                        Err(err) => {
                            log::error!(
                                "Delete Payment Submission - Processor failed to delete payment for chat {} with payment {}: {}",
                                chat.id,
                                display_payment(&payment, 1),
                                err.to_string()
                            );
                            bot.edit_message_text(
                                chat.id,
                                id,
                                format!("{}\nPayment deletion failed!", err.to_string()),
                            )
                            .await?;
                            dialogue
                                .update(State::ViewPayments { payments, page })
                                .await?;
                        }
                    }
                }
                _ => {
                    log::error!(
                        "Delete Payment Menu - Invalid button in chat {}: {}",
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
