use teloxide::{
    prelude::*,
    types::{Message, MessageId},
};

use crate::bot::{
    dispatcher::State,
    handler::{
        constants::{COMMAND_CANCEL, COMMAND_VIEW_PAYMENTS},
        utils::{
            display_balance_header, display_balances, display_payment, make_keyboard,
            send_bot_message, HandlerResult, UserDialogue,
        },
        Payment,
    },
    processor::delete_payment,
};

use super::utils::{assert_handle_request_limit, retrieve_time_zone};

/* Utilities */

const CANCEL_MESSAGE: &str =
    "Okay! I've cancelled deleting the payment. No changes have been made! 🌟";

/* Action handler functions */

/* Handles a repeated call to delete payment entry.
 * Does nothing, simply notifies the user.
 */
pub async fn handle_repeated_delete_payment(bot: Bot, msg: Message) -> HandlerResult {
    send_bot_message(
        &bot,
        &msg,
        format!("🚫 Oops! It seems like you're already in the middle of deleting a payment! Please finish or {COMMAND_CANCEL} this before starting another one with me."),
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
    state: State,
) -> HandlerResult {
    send_bot_message(&bot, &msg, CANCEL_MESSAGE.to_string()).await?;

    match state {
        State::SelectPayment {
            payments,
            page,
            function: _,
        }
        | State::DeletePayment {
            payment: _,
            payments,
            page,
        } => {
            dialogue
                .update(State::ViewPayments { payments, page })
                .await?;
        }
        _ => {
            dialogue.exit().await?;
        }
    }

    Ok(())
}

/* Blocks user command.
 * Called when user attempts to start another operation in the middle of deleting a payment.
 */
pub async fn block_delete_payment(bot: Bot, msg: Message) -> HandlerResult {
    send_bot_message(
        &bot,
        &msg,
        format!("🚫 Oops! It seems like you're in the middle of deleting a payment! Please finish or {COMMAND_CANCEL} this before starting something new with me."),
        ).await?;
    Ok(())
}

/* Blocks user command.
 * Called when user attempts to delete payment without first viewing anything.
 */
pub async fn no_delete_payment(bot: Bot, msg: Message) -> HandlerResult {
    if !assert_handle_request_limit(msg.clone()) {
        return Ok(());
    }

    send_bot_message(
        &bot,
        &msg,
        format!("Uh-oh! ❌ Sorry, please {COMMAND_VIEW_PAYMENTS} before deleting them!"),
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
    msg: &Message,
    msg_id: MessageId,
    (payments, page): (Vec<Payment>, usize),
    index: usize,
) -> HandlerResult {
    let payment = payments[index].clone();
    let keyboard = make_keyboard(vec!["Cancel", "Confirm"], Some(2));
    let chat_id = msg.chat.id.to_string();
    let time_zone = retrieve_time_zone(&chat_id);

    bot.edit_message_text(
        chat_id,
        msg_id,
        format!(
            "Do you really, really, want to 🗑 delete this payment? I won't be able to undo this... 🫢\n\n{}",
            display_payment(&payment, index + 1, time_zone)
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
            let chat_id = chat.id.to_string();
            let time_zone = retrieve_time_zone(&chat_id);
            match button.as_str() {
                "Cancel" => {
                    bot.edit_message_text(chat_id, id, format!("{CANCEL_MESSAGE}"))
                        .await?;
                    dialogue
                        .update(State::ViewPayments { payments, page })
                        .await?;
                }
                "Confirm" => {
                    let payment_id = &payment.payment_id;
                    let deletion = delete_payment(&chat_id, payment_id).await;

                    match deletion {
                        Ok(balances) => {
                            bot.edit_message_text(
                                chat_id.clone(),
                                id,
                                format!(
                                    "🎉 I've deleted the payment! 🎉\n\n{}{}",
                                    display_balance_header(&chat_id, &payment.currency.0),
                                    display_balances(&balances),
                                ),
                            )
                            .await?;

                            // Logging
                            log::info!(
                                "Delete Payment Submission - payment deleted for chat {} with payment {}",
                                chat_id,
                                display_payment(&payment, 1, time_zone)
                                );

                            dialogue
                                .update(State::ViewPayments { payments, page })
                                .await?;
                        }
                        Err(err) => {
                            bot.edit_message_text(
                                chat_id.clone(),
                                id,
                                format!("⁉️ Oh no! Something went wrong! 🥺 I'm sorry, but I can't delete the payment right now. Please try again later!\n\n" ),
                                )
                                .await?;

                            // Logging
                            log::error!(
                                "Delete Payment Submission - Processor failed to delete payment for chat {} with payment {}: {}",
                                chat_id,
                                display_payment(&payment, 1, time_zone),
                                err.to_string()
                                );

                            dialogue
                                .update(State::ViewPayments { payments, page })
                                .await?;
                        }
                    }
                }
                _ => {
                    log::error!(
                        "Delete Payment Menu - Invalid button in chat {}: {}",
                        chat_id,
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
