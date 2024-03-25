use teloxide::{payloads::SendMessageSetters, prelude::*, types::Message};

use crate::bot::{
    dispatcher::State,
    handler::{
        utils::{
            display_balances, display_payment, make_keyboard, parse_serial_num, BotError,
            HandlerResult, UserDialogue, COMMAND_VIEW_PAYMENTS,
        },
        Payment,
    },
    processor::delete_payment,
};

/* Utilities */

const CANCEL_MESSAGE: &str =
    "Sure, I've cancelled deleting the payment. No changes have been made! 👌";

/* Action handler functions */

/* Handles a repeated call to delete payment entry.
 * Does nothing, simply notifies the user.
 */
pub async fn handle_repeated_delete_payment(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(
        msg.chat.id,
        "🚫 You are already deleting a payment entry! Please complete or cancel the current operation before starting a new one.",
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
    bot.send_message(msg.chat.id, CANCEL_MESSAGE).await?;
    dialogue.exit().await?;
    Ok(())
}

/* Blocks user command.
 * Called when user attempts to start another operation in the middle of adding a payment.
 */
pub async fn block_delete_payment(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(
        msg.chat.id,
        "🚫 You are currently deleting a payment entry! Please complete or cancel the current payment entry before starting another command.",
    ).await?;
    Ok(())
}

/* Blocks user command.
 * Called when user attempts to delete payment without first viewing anything.
 */
pub async fn no_delete_payment(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(
        msg.chat.id,
        format!("❌ Please view the payment records first with {COMMAND_VIEW_PAYMENTS}!"),
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
        "Delete Payment - User not found in message: {}",
        msg.id.to_string()
    );
    Ok(())
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
        bot.answer_callback_query(query.id.to_string()).await?;

        if let Some(Message { id, chat, .. }) = query.message {
            match button.as_str() {
                "Cancel" => {
                    bot.edit_message_text(chat.id, id, format!("{CANCEL_MESSAGE}"))
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
                                    "🎉 I've deleted the payment!\n\nHere are the updated balances:\n{}",
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
                                format!("❓ Hmm, Something went wrong! Sorry, I can't delete the payment right now." ),
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
