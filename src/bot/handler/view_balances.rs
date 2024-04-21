use teloxide::{prelude::*, types::Message};

use crate::bot::{
    handler::{
        constants::UNKNOWN_ERROR_MESSAGE,
        utils::{display_balances, HandlerResult},
    },
    processor::view_debts,
};

/* Utilities */

/* View the balances for the group.
*/
pub async fn action_view_balances(bot: Bot, msg: Message) -> HandlerResult {
    let chat_id = msg.chat.id.to_string();
    if let Some(user) = msg.from() {
        let sender_id = user.id.to_string();
        let sender_username = user.username.clone();
        let debts = view_debts(&chat_id, &sender_id, sender_username.as_deref());
        match debts {
            Ok(debts) => {
                if debts.is_empty() {
                    bot.send_message(
                        msg.chat.id,
                        format!("There are no outstanding balances at the moment."),
                    )
                    .await?;

                    // Logging
                    log::info!("View Balances - User {} viewed balances for group {}, but no balances were recorded.", sender_id, chat_id);
                } else {
                    bot.send_message(
                        msg.chat.id,
                        format!(
                            "Here you go! The current balances are:\n\n{}",
                            display_balances(&debts, &chat_id)
                        ),
                    )
                    .await?;

                    // Logging
                    log::info!(
                        "View Balances - User {} viewed balances for group {}, found: {}",
                        sender_id,
                        chat_id,
                        display_balances(&debts, &chat_id)
                    );
                }
            }
            Err(err) => {
                bot.send_message(msg.chat.id, format!("{UNKNOWN_ERROR_MESSAGE}"))
                    .await?;

                // Logging
                log::error!(
                    "View Balances - User {} failed to view balances for group {}: {}",
                    sender_id,
                    chat_id,
                    err.to_string()
                );
            }
        }
    }

    Ok(())
}
