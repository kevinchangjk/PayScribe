use teloxide::{prelude::*, types::Message};

use crate::bot::{processor::view_debts, BotError};

use super::{
    super::dispatcher::{HandlerResult, UserDialogue},
    general::UNKNOWN_ERROR_MESSAGE,
    utils::display_balances,
};

/* Utilities */

/* View the balances for the group.
 */
pub async fn action_view_balances(bot: Bot, dialogue: UserDialogue, msg: Message) -> HandlerResult {
    let chat_id = msg.chat.id.to_string();
    if let Some(user) = msg.from() {
        let sender_id = user.id.to_string();
        let sender_username = user.username.clone();
        let debts = view_debts(&chat_id, &sender_id, sender_username.as_deref());
        match debts {
            Ok(debts) => {
                if debts.is_empty() {
                    log::info!("View Balances - User {} viewed balances for group {}, but no balances found.", sender_id, chat_id);
                    bot.send_message(msg.chat.id, format!("No balances found for this group!"))
                        .await?;
                } else {
                    log::info!(
                        "View Balances - User {} viewed balances for group {}, found: {}",
                        sender_id,
                        chat_id,
                        display_balances(&debts)
                    );
                    bot.send_message(
                        msg.chat.id,
                        format!(
                            "Here are the current balances for this group!\n\n{}",
                            display_balances(&debts)
                        ),
                    )
                    .await?;
                }
            }
            Err(err) => {
                log::error!(
                    "View Balances - User {} failed to view balances for group {}: {}",
                    sender_id,
                    chat_id,
                    err.to_string()
                );
                bot.send_message(
                    msg.chat.id,
                    format!(
                        "{}\nNo balances found for this group!",
                        UNKNOWN_ERROR_MESSAGE
                    ),
                )
                .await?;
            }
        }
        dialogue.exit().await?;
    }
    Err(BotError::UserError(
        "Unable to view balances: User not found".to_string(),
    ))
}
