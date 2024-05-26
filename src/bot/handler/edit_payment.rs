use teloxide::{
    payloads::SendMessageSetters,
    prelude::*,
    types::{Message, MessageId},
};

use crate::bot::{
    currency::Currency,
    dispatcher::State,
    handler::{
        constants::{
            COMMAND_CANCEL, COMMAND_VIEW_PAYMENTS, DEBT_EQUAL_DESCRIPTION_MESSAGE,
            DEBT_EQUAL_INSTRUCTIONS_MESSAGE, DEBT_EXACT_DESCRIPTION_MESSAGE,
            DEBT_EXACT_INSTRUCTIONS_MESSAGE, DEBT_RATIO_DESCRIPTION_MESSAGE,
            DEBT_RATIO_INSTRUCTIONS_MESSAGE, NO_TEXT_MESSAGE, TOTAL_INSTRUCTIONS_MESSAGE,
        },
        utils::{
            display_balance_header, display_balances, display_currency_amount, display_debts,
            display_payment, display_username, make_keyboard, make_keyboard_debt_selection,
            parse_currency_amount, parse_username, process_debts, retrieve_time_zone,
            send_bot_message, use_currency, HandlerResult, UserDialogue,
        },
        AddDebtsFormat, AddPaymentEdit, Payment,
    },
    processor::edit_payment,
};

use super::utils::{assert_handle_request_limit, delete_bot_messages};

/* Utilities */
#[derive(Clone, Debug)]
pub struct EditPaymentParams {
    description: Option<String>,
    creditor: Option<String>,
    currency: Option<Currency>,
    total: Option<i64>,
    debts: Option<Vec<(String, i64)>>,
}

const CANCEL_MESSAGE: &str = "Okay! I've cancelled the edit. No changes have been made! 🌟";

// Controls the state for misc handler actions that return to same state.
async fn repeat_state(
    dialogue: UserDialogue,
    state: State,
    new_message: MessageId,
) -> HandlerResult {
    match state {
        State::EditPayment {
            mut messages,
            payment,
            edited_payment,
            payments,
            page,
        } => {
            messages.push(new_message);
            dialogue
                .update(State::EditPayment {
                    messages,
                    payment,
                    edited_payment,
                    payments,
                    page,
                })
                .await?;
        }
        State::EditPaymentDetails {
            mut messages,
            payment,
            edited_payment,
            edit,
            payments,
            page,
        } => {
            messages.push(new_message);
            dialogue
                .update(State::EditPaymentDetails {
                    messages,
                    payment,
                    edited_payment,
                    edit,
                    payments,
                    page,
                })
                .await?;
        }
        State::EditPaymentDebtSelection {
            mut messages,
            payment,
            edited_payment,
            payments,
            page,
        } => {
            messages.push(new_message);
            dialogue
                .update(State::EditPaymentDebtSelection {
                    messages,
                    payment,
                    edited_payment,
                    payments,
                    page,
                })
                .await?;
        }
        _ => (),
    }

    Ok(())
}

// Controls the dialogue for ending a edit payment operation.
async fn complete_edit_payment(
    bot: &Bot,
    dialogue: UserDialogue,
    chat_id: &str,
    messages: Vec<MessageId>,
    payments: Vec<Payment>,
    page: usize,
) -> HandlerResult {
    delete_bot_messages(&bot, chat_id, messages).await?;
    dialogue
        .update(State::ViewPayments { payments, page })
        .await?;
    Ok(())
}

/* Displays a payment entry by combining original entry and edited fields.
*/
fn display_edit_payment(payment: Payment, edited_payment: EditPaymentParams) -> String {
    let currency = edited_payment.currency.unwrap_or(payment.currency);
    format!(
        "Description: {}\nPayer: {}\nTotal: {}\nSplit:\n{}",
        edited_payment.description.unwrap_or(payment.description),
        display_username(&edited_payment.creditor.unwrap_or(payment.creditor)),
        display_currency_amount(
            edited_payment.total.unwrap_or(payment.total),
            use_currency(currency.clone(), &payment.chat_id),
        ),
        display_debts(
            &edited_payment.debts.unwrap_or(payment.debts.clone()),
            currency.1
        )
    )
}

/* Edit a payment entry in a group chat.
 * Displays an overview of the current details provided.
 */
async fn display_edit_overview(
    bot: Bot,
    dialogue: UserDialogue,
    msg: &Message,
    msg_id: Option<MessageId>,
    mut messages: Vec<MessageId>,
    payment: Payment,
    edited_payment: EditPaymentParams,
    payments: Vec<Payment>,
    page: usize,
) -> HandlerResult {
    let options = vec![
        "Description",
        "Payer",
        "Total",
        "Split",
        "Cancel",
        "Confirm",
    ];
    let keyboard = make_keyboard(options, Some(2));
    match msg_id {
        Some(id) => {
            bot.edit_message_text(
                msg.chat.id,
                id,
                format!(
                    "Sure! What would you like to edit?\n\n{}",
                    display_edit_payment(payment.clone(), edited_payment.clone())
                ),
            )
            .reply_markup(keyboard)
            .await?;
        }
        None => {
            let new_message = send_bot_message(
                &bot,
                msg,
                format!(
                    "Sure! What would you like to edit?\n\n{}",
                    display_edit_payment(payment.clone(), edited_payment.clone())
                ),
            )
            .reply_markup(keyboard)
            .await?
            .id;
            messages.push(new_message);
        }
    }

    dialogue
        .update(State::EditPayment {
            messages,
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
    messages: Vec<MessageId>,
    payment: Payment,
    edited_payment: EditPaymentParams,
    payments: Vec<Payment>,
    page: usize,
    query: CallbackQuery,
) -> HandlerResult {
    if let Some(msg) = query.message {
        let chat_id = msg.chat.id.to_string();
        let edited_clone = edited_payment.clone();
        let user = msg.from();
        if let Some(user) = user {
            let edited = edit_payment(
                &chat_id,
                user.clone().username.unwrap_or("".to_string()),
                user.id.to_string(),
                &payment.payment_id,
                edited_payment.description.as_deref(),
                edited_payment.creditor.as_deref(),
                edited_payment.currency.clone().unzip().0.as_deref(),
                edited_payment.total.as_ref(),
                edited_payment.debts,
            )
            .await;

            match edited {
                Ok(balances) => {
                    let edit_overview = display_edit_payment(payment.clone(), edited_clone);
                    match balances {
                        Some(balances) => {
                            send_bot_message(
                                &bot,
                                &msg,
                                format!("🎉 Yay! Payment edited! 🎉\n\n{}", edit_overview,),
                            )
                            .await?;
                            send_bot_message(
                                &bot,
                                &msg,
                                format!(
                                    "{}{}",
                                    display_balance_header(
                                        &chat_id,
                                        edited_payment
                                            .currency
                                            .unzip()
                                            .0
                                            .as_deref()
                                            .unwrap_or(&payment.currency.0)
                                    ),
                                    display_balances(&balances)
                                ),
                            )
                            .await?;
                        }
                        None => {
                            send_bot_message(
                                &bot,
                                &msg,
                            format!(
                                "🎉 Yay! Payment edited! 🎉\n\n{}\nThere are no changes to the balances! 🥳",
                                edit_overview
                                ),
                                )
                            .await?;
                        }
                    }

                    // Logging
                    log::info!(
                        "Edit Payment Submission - payment edited for chat {} with payment {}",
                        chat_id,
                        edit_overview
                    );

                    delete_bot_messages(&bot, &chat_id, messages).await?;
                    dialogue
                        .update(State::ViewPayments { payments, page })
                        .await?;
                }
                Err(err) => {
                    let time_zone = retrieve_time_zone(&chat_id);
                    send_bot_message(
                        &bot,
                        &msg,
                        format!(
                            "⁉️ Oh no! Something went wrong! 🥺 I'm sorry, but I can't edit the payment right now. Please try again later!\n\n"
                        ),
                    )
                    .await?;

                    // Logging
                    log::error!(
                        "Edit Payment Submission - Processor failed to edit payment for chat {} with payment {}: {}",
                        chat_id,
                        display_payment(&payment, 1, time_zone),
                        err.to_string()
                    );

                    delete_bot_messages(&bot, &chat_id, messages).await?;
                    dialogue
                        .update(State::ViewPayments { payments, page })
                        .await?;
                }
            }
        }
    }

    Ok(())
}

/* Action handler functions */

/* Handles a repeated call to edit payment entry.
 * Does nothing, simply notifies the user.
 */
pub async fn handle_repeated_edit_payment(
    bot: Bot,
    dialogue: UserDialogue,
    state: State,
    msg: Message,
) -> HandlerResult {
    let new_message = send_bot_message(
        &bot,
        &msg,
        format!("🚫 Oops! It seems like you're already in the middle of editing a payment! Please finish or {COMMAND_CANCEL} this before starting another one with me."),
        ).await?;

    repeat_state(dialogue, state, new_message.id).await?;
    Ok(())
}

/* Cancels the edit payment operation.
 * Can be called at any step of the process.
 */
pub async fn cancel_edit_payment(
    bot: Bot,
    dialogue: UserDialogue,
    state: State,
    msg: Message,
) -> HandlerResult {
    send_bot_message(&bot, &msg, CANCEL_MESSAGE.to_string()).await?;

    match state {
        State::SelectPayment {
            messages,
            payments,
            page,
            function: _,
        }
        | State::EditPayment {
            messages,
            payment: _,
            edited_payment: _,
            payments,
            page,
        }
        | State::EditPaymentDebtSelection {
            messages,
            payment: _,
            edited_payment: _,
            payments,
            page,
        }
        | State::EditPaymentDetails {
            messages,
            payment: _,
            edited_payment: _,
            edit: _,
            payments,
            page,
        } => {
            complete_edit_payment(
                &bot,
                dialogue,
                &msg.chat.id.to_string(),
                messages,
                payments,
                page,
            )
            .await?;
        }
        _ => {
            dialogue.exit().await?;
        }
    }

    Ok(())
}

/* Blocks user command.
 * Called when user attempts to start another operation in the middle of adding a payment.
 */
pub async fn block_edit_payment(
    bot: Bot,
    dialogue: UserDialogue,
    state: State,
    msg: Message,
) -> HandlerResult {
    let new_message = send_bot_message(&bot,
        &msg,
        format!("🚫 Oops! It seems like you're in the middle of editing a payment! Please finish or {COMMAND_CANCEL} this before starting something new with me."),
        ).await?.id;

    repeat_state(dialogue, state, new_message).await?;
    Ok(())
}

/* Blocks user command.
 * Called when user attempts to edit payment without first viewing anything.
 */
pub async fn no_edit_payment(bot: Bot, msg: Message) -> HandlerResult {
    if !assert_handle_request_limit(msg.clone()) {
        return Ok(());
    }

    send_bot_message(
        &bot,
        &msg,
        format!("Uh-oh! ❌ Sorry, please {COMMAND_VIEW_PAYMENTS} before editing them!"),
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
    msg: &Message,
    msg_id: MessageId,
    (messages, payments, page): (Vec<MessageId>, Vec<Payment>, usize),
    index: usize,
) -> HandlerResult {
    let payment = payments[index].clone();
    let edited_payment = EditPaymentParams {
        description: None,
        creditor: None,
        currency: None,
        total: None,
        debts: None,
    };

    display_edit_overview(
        bot,
        dialogue,
        &msg,
        Some(msg_id),
        messages,
        payment,
        edited_payment,
        payments,
        page,
    )
    .await?;
    Ok(())
}

/* Edits a specified payment.
 * Bot receives a callback query to confirm the changes.
 */
pub async fn action_edit_payment_confirm(
    bot: Bot,
    dialogue: UserDialogue,
    state: State,
    (mut messages, payment, edited_payment, payments, page): (
        Vec<MessageId>,
        Payment,
        EditPaymentParams,
        Vec<Payment>,
        usize,
    ),
    query: CallbackQuery,
) -> HandlerResult {
    if let Some(button) = &query.data {
        bot.answer_callback_query(query.id.to_string()).await?;

        if let Some(msg) = &query.message {
            let chat_id = msg.chat.id.to_string();
            match button.as_str() {
                "Cancel" => {
                    cancel_edit_payment(bot, dialogue, state, msg.clone()).await?;
                }
                "Confirm" => {
                    call_processor_edit_payment(
                        bot,
                        dialogue,
                        messages,
                        payment,
                        edited_payment,
                        payments,
                        page,
                        query,
                    )
                    .await?;
                }
                "Description" => {
                    let new_message = send_bot_message(
                        &bot,
                        &msg,
                        format!(
                            "Current description: {}\n\nWhat should the description be?",
                            edited_payment
                                .description
                                .clone()
                                .unwrap_or(payment.description.clone())
                        ),
                    )
                    .await?
                    .id;
                    messages.push(new_message);
                    dialogue
                        .update(State::EditPaymentDetails {
                            messages,
                            payment,
                            edited_payment,
                            edit: AddPaymentEdit::Description,
                            payments,
                            page,
                        })
                        .await?;
                }
                "Payer" => {
                    let new_message = send_bot_message(
                        &bot,
                        &msg,
                        format!(
                            "Current payer: {}\n\nWho should the payer be?",
                            display_username(
                                &edited_payment
                                    .creditor
                                    .clone()
                                    .unwrap_or(payment.creditor.clone())
                            )
                        ),
                    )
                    .await?
                    .id;
                    messages.push(new_message);
                    dialogue
                        .update(State::EditPaymentDetails {
                            messages,
                            payment,
                            edited_payment,
                            edit: AddPaymentEdit::Creditor,
                            payments,
                            page,
                        })
                        .await?;
                }
                "Total" => {
                    let currency = edited_payment
                        .currency
                        .clone()
                        .unwrap_or(payment.currency.clone());
                    let actual_currency = use_currency(currency, &payment.chat_id);
                    let new_message = send_bot_message(
                        &bot,
                        &msg,
                        format!(
                            "Current total: {}\n\nWhat should the total be?\n\n{TOTAL_INSTRUCTIONS_MESSAGE}",
                            display_currency_amount(edited_payment.total.unwrap_or(payment.total), actual_currency)
                            ),
                            )
                        .await?.id;
                    messages.push(new_message);
                    dialogue
                        .update(State::EditPaymentDetails {
                            messages,
                            payment,
                            edited_payment,
                            edit: AddPaymentEdit::Total,
                            payments,
                            page,
                        })
                        .await?;
                }
                "Split" => {
                    let new_message = send_bot_message(
                        &bot,
                        &msg,
                        format!(
                            "Current split:\n{}\nHow should we split this?\n\n{DEBT_EQUAL_DESCRIPTION_MESSAGE}{DEBT_EXACT_DESCRIPTION_MESSAGE}{DEBT_RATIO_DESCRIPTION_MESSAGE}",
                            display_debts(&edited_payment.debts.clone().unwrap_or(payment.debts.clone()), edited_payment.currency.clone().unwrap_or(payment.currency.clone()).1)
                            )
                            ).reply_markup(make_keyboard_debt_selection())
                        .await?.id;
                    messages.push(new_message);
                    dialogue
                        .update(State::EditPaymentDebtSelection {
                            messages,
                            payment,
                            edited_payment,
                            payments,
                            page,
                        })
                        .await?;
                }
                _ => {
                    log::error!(
                        "Edit Payment Menu - Invalid button in chat {}: {}",
                        chat_id,
                        button
                    );
                }
            }
        }
    }

    Ok(())
}

/* Edits a specified payment.
 * Bot receives a callback query on how to specify changes to debts.
 */
pub async fn action_edit_payment_debts(
    bot: Bot,
    dialogue: UserDialogue,
    query: CallbackQuery,
    (messages, payment, edited_payment, payments, page): (
        Vec<MessageId>,
        Payment,
        EditPaymentParams,
        Vec<Payment>,
        usize,
    ),
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
                            "Okay! Who is involved in the payment?\n\n{DEBT_EQUAL_INSTRUCTIONS_MESSAGE}",
                            ),
                            )
                        .await?;
                    dialogue
                        .update(State::EditPaymentDetails {
                            messages,
                            payment,
                            edited_payment,
                            edit: AddPaymentEdit::DebtsEqual,
                            payments,
                            page,
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
                            "Okay! Who is involved and how much do they owe?\n\n{DEBT_EXACT_INSTRUCTIONS_MESSAGE}",
                            )).await?;
                    dialogue
                        .update(State::EditPaymentDetails {
                            messages,
                            payment,
                            edited_payment,
                            edit: AddPaymentEdit::DebtsExact,
                            payments,
                            page,
                        })
                        .await?;
                }
            }
            "Proportion" => {
                if let Some(Message { id, chat, .. }) = query.message {
                    bot.edit_message_text(
                        chat.id,
                        id,
                        format!(
                            "Okay! Who is involved and how much do they owe?\n\n{DEBT_RATIO_INSTRUCTIONS_MESSAGE}",
                            )).await?;
                    dialogue
                        .update(State::EditPaymentDetails {
                            messages,
                            payment,
                            edited_payment,
                            edit: AddPaymentEdit::DebtsRatio,
                            payments,
                            page,
                        })
                        .await?;
                }
            }
            _ => {
                log::error!("Edit Payment Debt Selection - Invalid button for in chat {} with payment {:?}: {}",
                            payment.chat_id, payment, button);
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
    (mut messages, payment, edited_payment, edit, payments, page): (
        Vec<MessageId>,
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
                    currency: edited_payment.currency,
                    total: edited_payment.total,
                    debts: edited_payment.debts,
                };
                display_edit_overview(
                    bot,
                    dialogue,
                    &msg,
                    None,
                    messages,
                    payment,
                    new_edited_payment,
                    payments,
                    page,
                )
                .await?;
            }
            AddPaymentEdit::Creditor => {
                let username = parse_username(text);
                if let Err(err) = username {
                    let new_message = send_bot_message(&bot, &msg, err.to_string()).await?.id;
                    messages.push(new_message);
                    dialogue
                        .update(State::EditPaymentDetails {
                            messages,
                            payment,
                            edited_payment,
                            edit,
                            payments,
                            page,
                        })
                        .await?;
                    return Ok(());
                }
                let new_edited_payment = EditPaymentParams {
                    description: edited_payment.description,
                    creditor: Some(username?),
                    currency: edited_payment.currency,
                    total: edited_payment.total,
                    debts: edited_payment.debts,
                };
                display_edit_overview(
                    bot,
                    dialogue,
                    &msg,
                    None,
                    messages,
                    payment,
                    new_edited_payment,
                    payments,
                    page,
                )
                .await?;
            }
            AddPaymentEdit::Total => {
                let currency_amount = parse_currency_amount(text);
                match currency_amount {
                    Ok((total, currency)) => {
                        let new_edited_payment = EditPaymentParams {
                            description: edited_payment.description,
                            creditor: edited_payment.creditor,
                            currency: Some(currency),
                            total: Some(total),
                            debts: None,
                        };

                        let new_message = send_bot_message(
                            &bot,
                            &msg,
                            format!("Fantastic! How should we split this?\n\n{DEBT_EQUAL_DESCRIPTION_MESSAGE}{DEBT_EXACT_DESCRIPTION_MESSAGE}{DEBT_RATIO_DESCRIPTION_MESSAGE}")
                            )
                            .reply_markup(make_keyboard_debt_selection())
                            .await?.id;
                        messages.push(new_message);
                        dialogue
                            .update(State::EditPaymentDebtSelection {
                                messages,
                                payment,
                                edited_payment: new_edited_payment,
                                payments,
                                page,
                            })
                            .await?;
                    }
                    Err(err) => {
                        let new_message = send_bot_message(
                            &bot,
                            &msg,
                            format!(
                                "{}\n\nWhat should the total be?\n\n{TOTAL_INSTRUCTIONS_MESSAGE}",
                                err.to_string()
                            ),
                        )
                        .await?
                        .id;
                        messages.push(new_message);
                        dialogue
                            .update(State::EditPaymentDetails {
                                messages,
                                payment,
                                edited_payment,
                                edit,
                                payments,
                                page,
                            })
                            .await?;
                        return Ok(());
                    }
                }
            }
            AddPaymentEdit::DebtsEqual
            | AddPaymentEdit::DebtsExact
            | AddPaymentEdit::DebtsRatio => {
                let debts_format = match edit {
                    AddPaymentEdit::DebtsEqual => AddDebtsFormat::Equal,
                    AddPaymentEdit::DebtsExact => AddDebtsFormat::Exact,
                    AddPaymentEdit::DebtsRatio => AddDebtsFormat::Ratio,
                    _ => AddDebtsFormat::Equal,
                };
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
                            &edited_payment
                                .creditor
                                .clone()
                                .or(Some(payment.creditor.clone())),
                            edited_payment
                                .currency
                                .clone()
                                .or(Some(payment.currency.clone())),
                            edited_payment.total.or(Some(payment.total)),
                        );
                        if let Err(err) = debts {
                            let new_message = send_bot_message(
                                &bot,
                                &msg,
                                format!("{}\n\n{error_msg}", err.to_string()),
                            )
                            .await?
                            .id;
                            messages.push(new_message);
                            dialogue
                                .update(State::EditPaymentDetails {
                                    messages,
                                    payment,
                                    edited_payment,
                                    edit,
                                    payments,
                                    page,
                                })
                                .await?;
                            return Ok(());
                        }

                        let new_edited_payment = EditPaymentParams {
                            description: edited_payment.description,
                            creditor: edited_payment.creditor,
                            currency: edited_payment.currency,
                            total: edited_payment.total,
                            debts: Some(debts.unwrap()),
                        };

                        display_edit_overview(
                            bot,
                            dialogue,
                            &msg,
                            None,
                            messages,
                            payment,
                            new_edited_payment,
                            payments,
                            page,
                        )
                        .await?;
                    }
                    None => {
                        send_bot_message(&bot, &msg, format!("{error_msg}")).await?;
                    }
                }
            }
        },
        None => {
            send_bot_message(&bot, &msg, format!("{NO_TEXT_MESSAGE}")).await?;
        }
    }

    Ok(())
}
