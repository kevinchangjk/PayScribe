use teloxide::{
    payloads::SendMessageSetters,
    prelude::*,
    types::{InlineKeyboardButton, InlineKeyboardMarkup, Message},
};

use crate::bot::{handler::utils::display_debts, processor::add_payment};

use super::{
    super::dispatcher::{HandlerResult, State, UserDialogue},
    general::UNKNOWN_ERROR_MESSAGE,
    utils::{display_balances, make_keyboard, parse_amount, parse_debts, parse_username},
};

/* Utilities */
const HEADER_MESSAGE: &str = "Adding a new payment entry!\n\n";
const FOOTER_MESSAGE: &str = "\n\n";
const NO_TEXT_MESSAGE: &str = "Please reply in text.\n\n";
const DEBT_INSTRUCTIONS_MESSAGE: &str =
    "Enter the usernames and the amounts as follows: \n\n@user1 amount1, @user2 amount2, etc.\n\n";

#[derive(Clone, Debug)]
pub struct AddPaymentParams {
    chat_id: String,
    sender_id: String,
    sender_username: Option<String>,
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
    chat_id: String,
    payment: AddPaymentParams,
) -> HandlerResult {
    let buttons = vec!["Cancel", "Edit", "Confirm"];
    let keyboard = make_keyboard(buttons, Some(3));

    bot.send_message(chat_id, display_add_payment(&payment))
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
            let debts = parse_debts(text, &payment.creditor, payment.total);
            if let Err(err) = debts {
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
            display_add_overview(bot, dialogue, msg.chat.id.to_string(), new_payment).await?;
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
                bot.edit_message_text(chat.id, id, UNKNOWN_ERROR_MESSAGE)
                    .await?;
                dialogue.exit().await?;
                return Ok(());
            }
        };
        let creditor = match payment.creditor {
            Some(cred) => cred,
            None => {
                bot.edit_message_text(chat.id, id, UNKNOWN_ERROR_MESSAGE)
                    .await?;
                dialogue.exit().await?;
                return Ok(());
            }
        };
        let total = match payment.total {
            Some(tot) => tot,
            None => {
                bot.edit_message_text(chat.id, id, UNKNOWN_ERROR_MESSAGE)
                    .await?;
                dialogue.exit().await?;
                return Ok(());
            }
        };
        let debts = match payment.debts {
            Some(debts) => debts,
            None => {
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
        if let Err(err) = updated_balances {
            bot.edit_message_text(
                chat.id,
                id,
                format!("{}\nPayment entry failed!", err.to_string()),
            )
            .await?;
        } else {
            bot.edit_message_text(
                chat.id,
                id,
                format!(
                    "Payment entry added!\n\n{}Current balances:\n{}",
                    payment_overview,
                    display_balances(&updated_balances?)
                ),
            )
            .await?;
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
        "You are currently adding a payment entry! Please complete or cancel the current payment entry before another command.",
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
                let payment = AddPaymentParams {
                    chat_id: msg.chat.id.to_string(),
                    sender_id: user.id.to_string(),
                    sender_username: user.username.clone(),
                    datetime: msg.date.to_string(),
                    description: Some(text.to_string()),
                    creditor: None,
                    total: None,
                    debts: None,
                };
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
            _ => {}
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
            match button.as_str() {
                "Description" => {
                    bot.edit_message_text(
                        chat.id,
                        id,
                        format!(
                            "Current description: {}\n\nWhat do you want the new description to be?",
                            payment.clone().description.unwrap()
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
                            payment.clone().description.unwrap()
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
                            payment.clone().description.unwrap()
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
                            display_debts(&payment.clone().debts.unwrap())
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
                    display_add_overview(bot, dialogue, chat.id.to_string(), payment).await?;
                }
                _ => {}
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
                display_add_overview(bot, dialogue, msg.chat.id.to_string(), new_payment).await?;
            }
            AddPaymentEdit::Creditor => {
                let new_payment = AddPaymentParams {
                    chat_id: payment.chat_id,
                    sender_id: payment.sender_id,
                    sender_username: payment.sender_username,
                    datetime: payment.datetime,
                    description: payment.description,
                    creditor: Some(text.to_string()),
                    total: payment.total,
                    debts: payment.debts,
                };
                display_add_overview(bot, dialogue, msg.chat.id.to_string(), new_payment).await?;
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
