use teloxide::{
    payloads::SendMessageSetters,
    prelude::*,
    types::{InlineKeyboardButton, InlineKeyboardMarkup, Message},
};

use super::super::dispatcher::{AddPaymentParams, HandlerResult, State, UserDialogue};

/* Add a payment entry in a group chat.
 * Bot will ask for user to send messages to fill in required information,
 * before presenting the compiled information for confirmation with a menu.
 */
pub async fn action_add_payment(bot: Bot, dialogue: UserDialogue, msg: Message) -> HandlerResult {
    bot.send_message(
        msg.chat.id,
        "Adding a new payment entry!\nEnter a description for the payment: ",
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
            let payment = AddPaymentParams {
                description: Some(text.to_string()),
                creditor: None,
                total: None,
                debts: None,
            };
            bot.send_message(
                msg.chat.id,
                "Adding a new payment entry!\nEnter the username of the one who paid the total: ",
            )
            .await?;
            dialogue.update(State::AddCreditor { payment }).await?;
        }
        None => {
            bot.send_message(
                msg.chat.id,
                "Please enter the description for the payment in text: ",
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
            // Ensures that the username starts with '@'
            let text: String = if text.chars().next() == Some('@') {
                text.to_string()
            } else {
                format!("@{}", text)
            };

            let new_payment = AddPaymentParams {
                description: payment.description,
                creditor: Some(text),
                total: None,
                debts: None,
            };
            bot.send_message(
                msg.chat.id,
                "Adding a new payment entry!\nEnter the total amount paid (without currency): ",
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
                "Please enter the username of the payer in text: ",
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
            // Check if can parse the text to f64
            let total = match text.parse::<f64>() {
                Ok(val) => val,
                Err(_) => {
                    if let Ok(val) = text.parse::<i32>() {
                        val as f64
                    } else {
                        bot.send_message(
                            msg.chat.id,
                            "Please enter the total amount paid as a valid number without any symbols: ",
                        )
                        .await?;
                        return Ok(());
                    }
                }
            };

            let new_payment = AddPaymentParams {
                description: payment.description,
                creditor: payment.creditor,
                total: Some(total),
                debts: None,
            };
            bot.send_message(
                msg.chat.id,
                "Adding a new payment entry!\nWho are we splitting this with? Enter the username and the amount (without currency) as follows: \n\nUSERNAME AMOUNT",
            )
            .await?;
            dialogue
                .update(State::AddDebt {
                    payment: new_payment,
                })
                .await?;
        }
        None => {
            bot.send_message(msg.chat.id, "Please enter the total amount paid in text: ")
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
    match msg.text() {
        Some(text) => {
            // Parse the text to get username and amount
            let text: Vec<&str> = text.split(' ').collect();
            if text.len() != 2 {
                bot.send_message(
                    msg.chat.id,
                    "Please enter the username and the amount (without currency) as follows: \n\nUSERNAME AMOUNT",
                )
                .await?;
                return Ok(());
            }

            // Ensures that the username starts with '@'
            let username: String = if text[0].chars().next() == Some('@') {
                text[0].to_string()
            } else {
                format!("@{}", text[0])
            };

            // Check if can parse the text to f64
            let amount = match text[1].parse::<f64>() {
                Ok(val) => val,
                Err(_) => {
                    if let Ok(val) = text[1].parse::<i32>() {
                        val as f64
                    } else {
                        bot.send_message(
                            msg.chat.id,
                            "Please enter the amount as a valid number without any symbols: ",
                        )
                        .await?;
                        return Ok(());
                    }
                }
            };

            let new_debts = match payment.debts {
                Some(mut debts) => {
                    debts.push((username, amount));
                    Some(debts)
                }
                None => Some(vec![(username, amount)]),
            };

            let sum = new_debts
                .as_ref()
                .unwrap()
                .iter()
                .fold(0.0, |acc, (_, amount)| acc + amount);

            let new_payment = AddPaymentParams {
                description: payment.description,
                creditor: payment.creditor,
                total: payment.total,
                debts: new_debts,
            };

            if sum != payment.total.unwrap() {
                bot.send_message(
                    msg.chat.id,
                    format!(
                "Who else are we splitting this with? Please enter the username and the amount (without currency) as follows: \n\nUSERNAME AMOUNT\n\nTotal amount paid: {}\nTotal amount of debts: {}",
                payment.total.unwrap(),
                sum
                    ),
                )
                .await?;
                dialogue
                    .update(State::AddDebt {
                        payment: new_payment,
                    })
                    .await?;
                return Ok(());
            }

            display_add_overview(bot, dialogue, msg.chat.id.to_string(), new_payment).await?;
        }
        None => {
            bot.send_message(msg.chat.id, "Please enter the username and the amount (without symbols) as follows: \n\nUSERNAME AMOUNT")
                .await?;
        }
    }
    Ok(())
}

/* Add a payment entry in a group chat.
 * Displays an overview of the current details provided.
 * Is not a normal endpoint function, just a temporary transition function.
 */
pub async fn display_add_overview(
    bot: Bot,
    dialogue: UserDialogue,
    chat_id: String,
    payment: AddPaymentParams,
) -> HandlerResult {
    let payment_clone = payment.clone();
    let keyboard: Vec<Vec<InlineKeyboardButton>> = vec![["Cancel", "Edit", "Confirm"]
        .iter()
        .map(|&button| InlineKeyboardButton::callback(button.to_owned(), button.to_owned()))
        .collect()];

    bot.send_message(
        chat_id,
                format!(
                    "Overview of the payment entry:\n\nDescription: {}\nCreditor: {}\nTotal: {}\nDebts: {:?}",
                    payment.description.unwrap(),
                    payment.creditor.unwrap(),
                    payment.total.unwrap(),
                    payment.debts.unwrap()
                ),
            ).reply_markup(InlineKeyboardMarkup::new(keyboard))
            .await?;
    dialogue
        .update(State::AddConfirm {
            payment: payment_clone,
        })
        .await?;
    Ok(())
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
                if let Some(Message { id, chat, .. }) = query.message {
                    bot.edit_message_text(chat.id, id, "Payment entry added!")
                        .await?;
                    dialogue.exit().await?;
                }
            }
            _ => {}
        }
    }
    Ok(())
}

/* Add a payment entry in a group chat.
 * Displays a button menu for user to choose which part of the payment details to edit.
 */
pub async fn display_add_edit_menu(
    bot: Bot,
    dialogue: UserDialogue,
    payment: AddPaymentParams,
    query: CallbackQuery,
) -> HandlerResult {
    let buttons = ["Description", "Creditor", "Total", "Debts", "Back"];

    let mut keyboard: Vec<Vec<InlineKeyboardButton>> = vec![];
    for pair in buttons.chunks(2) {
        let row = pair
            .iter()
            .map(|&button| InlineKeyboardButton::callback(button.to_owned(), button.to_owned()))
            .collect();
        keyboard.push(row);
    }

    if let Some(Message { id, chat, .. }) = query.message {
        bot.edit_message_text(
            chat.id,
            id,
            "Which part of the payment details would you like to edit?",
        )
        .reply_markup(InlineKeyboardMarkup::new(keyboard))
        .await?;
        dialogue.update(State::AddEdit { payment }).await?;
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
    payment: AddPaymentParams,
    query: CallbackQuery,
) -> HandlerResult {
    if let Some(button) = &query.data {
        bot.answer_callback_query(format!("{}", query.id)).await?;

        if let Some(Message { id, chat, .. }) = query.message {
            match button.as_str() {
                /*
                "Description" => {
                    bot.send_message(
                        msg.chat.id,
                        "Enter a new description for the payment: ",
                    )
                    .await?;
                    dialogue.update(State::AddDescription).await?;
                }
                "Creditor" => {
                    bot.send_message(
                        msg.chat.id,
                        "Enter the username of the one who paid the total: ",
                    )
                    .await?;
                    dialogue.update(State::AddCreditor { payment }).await?;
                }
                "Total" => {
                    bot.send_message(
                        msg.chat.id,
                        "Enter the total amount paid (without currency): ",
                    )
                    .await?;
                    dialogue.update(State::AddTotal { payment }).await?;
                }
                "Debts" => {
                    bot.send_message(
                        msg.chat.id,
                        "Who are we splitting this with? Enter the username and the amount (without currency) as follows: \n\nUSERNAME AMOUNT",
                    )
                    .await?;
                    dialogue.update(State::AddDebt { payment }).await?;
                }
                */
                "Back" => {
                    display_add_overview(bot, dialogue, chat.id.to_string(), payment).await?;
                }
                _ => {}
            }
        }
    }

    Ok(())
}