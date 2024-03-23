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

#[derive(Clone, Debug)]
pub struct Payment {
    payment_id: String,
    chat_id: String,
    datetime: String,
    description: Option<String>,
    creditor: Option<String>,
    total: Option<f64>,
    debts: Option<Vec<(String, f64)>>,
}

pub async fn action_view_payments(bot: Bot, dialogue: UserDialogue, msg: Message) -> HandlerResult {
    Ok(())
}

pub async fn action_view_more(bot: Bot, dialogue: UserDialogue, msg: Message) -> HandlerResult {
    Ok(())
}
