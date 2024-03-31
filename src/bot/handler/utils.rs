use chrono::{DateTime, Local, NaiveDateTime};
use regex::Regex;
use teloxide::{
    dispatching::dialogue::{Dialogue, InMemStorage, InMemStorageError},
    types::{InlineKeyboardButton, InlineKeyboardMarkup},
    RequestError,
};

use crate::bot::{processor::ProcessError, redis::Debt, State};

use super::{AddDebtsFormat, Payment};

/* Common utilites for handlers. */

pub const MAX_VALUE: f64 = 10_000_000_000_000.00;
pub const UNKNOWN_ERROR_MESSAGE: &str =
    "❓ Hmm, something went wrong! Sorry, I can't do that right now, please try again later!\n\n";
pub const NO_TEXT_MESSAGE: &str =
    "❓ Sorry, I can't understand that! Please reply to me in text.\n\n";
pub const DEBT_EQUAL_DESCRIPTION_MESSAGE: &str =
    "Equal — Divide the total amount equally among users\n";
pub const DEBT_EXACT_DESCRIPTION_MESSAGE: &str =
    "Exact — Share the total cost by specifying exact amounts for each user\n";
pub const DEBT_RATIO_DESCRIPTION_MESSAGE: &str =
    "Ratio — Split the total cost by assigning fractional/relative amounts of the total that each user owes\n";
pub const DEBT_EQUAL_INSTRUCTIONS_MESSAGE: &str =
    "Enter the usernames of those sharing the cost (including the payer if sharing too) as follows: \n\n@user1 @user2 @user3 ...\n\n";
pub const DEBT_EXACT_INSTRUCTIONS_MESSAGE: &str =
    "Enter the usernames and exact amounts as follows: \n\n@user1 amount1\n@user2 amount2\n@user3 amount3\n...\n\nAny leftover amount will be taken as the payer's share.";
pub const PAY_BACK_INSTRUCTIONS_MESSAGE: &str =
    "Enter the usernames and exact amounts as follows: \n\n@user1 amount1\n@user2 amount2\n@user3 amount3\n...\n\n";
pub const DEBT_RATIO_INSTRUCTIONS_MESSAGE: &str =
    "Enter the usernames and ratios as follows: \n\n@user1 ratio1\n@user2 ratio2\n@user3 ratio3\n...\n\nThe ratios can be any number, and do not need to sum up to any specific number.";
pub const COMMAND_START: &str = "/start";
pub const COMMAND_HELP: &str = "/help";
pub const COMMAND_ADD_PAYMENT: &str = "/addpayment";
pub const COMMAND_PAY_BACK: &str = "/payback";
pub const COMMAND_VIEW_PAYMENTS: &str = "/viewpayments";
pub const COMMAND_EDIT_PAYMENT: &str = "/editpayment";
pub const COMMAND_DELETE_PAYMENT: &str = "/deletepayment";
pub const COMMAND_VIEW_BALANCES: &str = "/viewbalances";

/* Types */
pub type UserDialogue = Dialogue<State, InMemStorage<State>>;
pub type HandlerResult = Result<(), BotError>;

#[derive(Debug, Clone)]
pub enum SelectPaymentType {
    EditPayment,
    DeletePayment,
}

#[derive(thiserror::Error, Debug)]
pub enum BotError {
    #[error("{0}")]
    UserError(String),
    #[error("Process error: {0}")]
    ProcessError(ProcessError),
    #[error("Request error: {0}")]
    RequestError(RequestError),
}

impl From<RequestError> for BotError {
    fn from(request_error: RequestError) -> BotError {
        BotError::RequestError(request_error)
    }
}

impl From<InMemStorageError> for BotError {
    fn from(storage_error: InMemStorageError) -> BotError {
        BotError::UserError(storage_error.to_string())
    }
}

impl From<ProcessError> for BotError {
    fn from(process_error: ProcessError) -> BotError {
        BotError::ProcessError(process_error)
    }
}

// Displays balances in a more readable format.
pub fn display_balances(debts: &Vec<Debt>) -> String {
    let mut message = String::new();
    for debt in debts {
        message.push_str(&format!(
            "{} owes {}: {:.2}\n",
            debt.debtor, debt.creditor, debt.amount
        ));
    }

    if message.is_empty() {
        "No outstanding balances! 👍".to_string()
    } else {
        message
    }
}

// Displays debts in a more readable format.
pub fn display_debts(debts: &Vec<(String, f64)>) -> String {
    let mut message = String::new();
    for debt in debts {
        message.push_str(&format!("    {}: {:.2}\n", debt.0, debt.1));
    }
    message
}

// Displays a single payment entry in a user-friendly format.
pub fn display_payment(payment: &Payment, serial_num: usize) -> String {
    format!(
        "__________________________\n{}. {}\nDate: {}\nPayer: {}\nTotal: {:.2}\n{}",
        serial_num,
        payment.description,
        reformat_datetime(&payment.datetime),
        payment.creditor,
        payment.total,
        display_debts(&payment.debts)
    )
}

// Make a keyboard, button menu.
pub fn make_keyboard(options: Vec<&str>, columns: Option<usize>) -> InlineKeyboardMarkup {
    let mut keyboard: Vec<Vec<InlineKeyboardButton>> = Vec::new();
    if let Some(col) = columns {
        for chunk in options.chunks(col) {
            let mut row: Vec<InlineKeyboardButton> = Vec::new();
            for option in chunk {
                row.push(InlineKeyboardButton::callback(
                    option.to_string(),
                    option.to_string(),
                ));
            }
            keyboard.push(row);
        }
    } else {
        for option in options {
            keyboard.push(vec![InlineKeyboardButton::callback(option, option)]);
        }
    }

    InlineKeyboardMarkup::new(keyboard)
}

// Make debt selection keyboard
pub fn make_keyboard_debt_selection() -> InlineKeyboardMarkup {
    let buttons = vec!["Equal", "Exact", "Ratio"];
    make_keyboard(buttons, Some(3))
}

// Ensures that a username has a leading '@'.
pub fn parse_username(username: &str) -> Result<String, BotError> {
    let text: &str;
    if username.starts_with('@') {
        text = username.trim_start_matches('@');
    } else {
        text = username;
    }

    if text.split_whitespace().count() == 1 && text.len() >= 5 {
        let re = Regex::new(r"^[a-zA-Z0-9_]+$");
        if let Ok(re) = re {
            if re.captures(text).is_some() {
                return Ok(format!("@{}", text.to_string()));
            }
        }
    }

    Err(BotError::UserError(
        "❌ Please provide a valid username!".to_string(),
    ))
}

// Parse an amount. Reads a string, returns f64.
pub fn parse_amount(text: &str) -> Result<f64, BotError> {
    let text = text
        .chars()
        .filter(|&c| c.is_digit(10) || c == '.')
        .collect::<String>();

    let amount = match text.parse::<f64>() {
        Ok(val) => val,
        Err(_) => match text.parse::<i32>() {
            Ok(val) => val as f64,
            Err(_) => {
                return Err(BotError::UserError(
                    "❌ Please provide a valid number!".to_string(),
                ))
            }
        },
    };

    if amount > MAX_VALUE {
        Err(BotError::UserError(
            "❌ This number is too large for me to process!".to_string(),
        ))
    } else if amount <= 0.0 {
        Err(BotError::UserError(
            "❌ Please provide a positive value!".to_string(),
        ))
    } else {
        Ok(amount)
    }
}

// Parse and process a string to retrieve a list of debts, for split by equal amount.
pub fn process_debts_equal(text: &str, total: Option<f64>) -> Result<Vec<(String, f64)>, BotError> {
    let users = text.split_whitespace().collect::<Vec<&str>>();
    if users.len() == 0 {
        return Err(BotError::UserError(
            "❌ Please provide at least one username!".to_string(),
        ));
    }

    let total = match total {
        Some(val) => val,
        None => {
            return Err(BotError::UserError(
                "❌ Something's wrong! The total amount isn't provided.".to_string(),
            ));
        }
    };

    let amount = total / users.len() as f64;
    let mut debts: Vec<(String, f64)> = Vec::new();
    for user in &users {
        let debt = (parse_username(user)?, amount);
        debts.push(debt);
    }

    Ok(debts)
}

// Parse and process a string to retrieve a list of debts, for split by exact amount.
pub fn process_debts_exact(
    text: &str,
    creditor: &Option<String>,
    total: Option<f64>,
) -> Result<Vec<(String, f64)>, BotError> {
    let mut debts: Vec<(String, f64)> = Vec::new();
    let mut sum: f64 = 0.0;
    let items: Vec<&str> = text.split_whitespace().collect();
    if items.len() % 2 != 0 {
        return Err(BotError::UserError(
            "❌ Please use the following format!".to_string(),
        ));
    }

    for i in (0..items.len()).step_by(2) {
        let username = parse_username(items[i])?;
        let amount = parse_amount(items[i + 1])?;
        sum += amount;

        let mut found = false;
        for debt in &mut debts {
            if debt.0 == username {
                debt.1 += amount;
                found = true;
                break;
            }
        }

        if !found {
            debts.push((username, amount));
        }
    }

    if let Some(creditor) = creditor {
        if let Some(total) = total {
            if sum > total {
                Err(BotError::UserError(
                    "❌ Something's wrong! The sum of the amounts exceeds the total paid."
                        .to_string(),
                ))
            } else if sum < total {
                for debt in &mut debts {
                    if debt.0 == creditor.to_string() {
                        debt.1 += total - sum;
                        return Ok(debts);
                    }
                }

                debts.push((creditor.to_string(), total - sum));
                Ok(debts)
            } else {
                Ok(debts)
            }
        } else {
            Err(BotError::UserError(
                "❌ Something's wrong! The total amount isn't provided.".to_string(),
            ))
        }
    } else {
        Err(BotError::UserError(
            "❌ Something's wrong! The payer isn't provided.".to_string(),
        ))
    }
}

// Parse and process a string to retrieve a list of debts, for split by ratio.
pub fn process_debts_ratio(text: &str, total: Option<f64>) -> Result<Vec<(String, f64)>, BotError> {
    let items: Vec<&str> = text.split_whitespace().collect();
    let mut debts: Vec<(String, f64)> = Vec::new();
    let mut sum: f64 = 0.0;

    if items.len() % 2 != 0 {
        return Err(BotError::UserError(
            "❌ Please use the following format!".to_string(),
        ));
    }

    for i in (0..items.len()).step_by(2) {
        let username = parse_username(items[i])?;
        let ratio = parse_amount(items[i + 1])?;

        sum += ratio;
        debts.push((username, ratio));
    }

    let total = match total {
        Some(val) => val,
        None => {
            return Err(BotError::UserError(
                "❌ Something's wrong! The total amount isn't provided.".to_string(),
            ));
        }
    };

    for debt in &mut debts {
        debt.1 = (debt.1 / sum) * total;
    }

    Ok(debts)
}

// Parse and process a string to retrieve a list of debts, returns Vec<Debt>.
pub fn process_debts(
    debts_format: AddDebtsFormat,
    text: &str,
    creditor: &Option<String>,
    total: Option<f64>,
) -> Result<Vec<(String, f64)>, BotError> {
    match debts_format {
        AddDebtsFormat::Equal => process_debts_equal(text, total),
        AddDebtsFormat::Exact => process_debts_exact(text, creditor, total),
        AddDebtsFormat::Ratio => process_debts_ratio(text, total),
    }
}

// Parses a string of debts and returns a vector of debts
pub fn parse_debts_payback(text: &str, sender: &str) -> Result<Vec<(String, f64)>, BotError> {
    let mut debts: Vec<(String, f64)> = Vec::new();
    let items: Vec<&str> = text.split_whitespace().collect();
    if items.len() % 2 != 0 {
        return Err(BotError::UserError(
            "❌ Please use the following format!".to_string(),
        ));
    }

    for i in (0..items.len()).step_by(2) {
        let username = parse_username(items[i])?;
        let amount = parse_amount(items[i + 1])?;
        if username == sender {
            return Err(BotError::UserError(
                "❌ You can't pay back yourself!".to_string(),
            ));
        }
        let mut found = false;
        for debt in &mut debts {
            if debt.0 == username {
                debt.1 += amount;
                found = true;
                break;
            }
        }
        if !found {
            debts.push((username, amount));
        }
    }

    Ok(debts)
}

// Parses a string representing a datetime, and returns the Datetime object
pub fn parse_datetime(text: &str) -> DateTime<Local> {
    // Checks if text contains "UTC" at the end
    let mut new_text = text.to_string();
    if text.ends_with(" UTC") {
        new_text = new_text.replace(" UTC", "");
    }
    let datetime = NaiveDateTime::parse_from_str(&new_text, "%Y-%m-%d %H:%M:%S");
    match datetime {
        Ok(val) => val.and_utc().with_timezone(&Local),
        Err(_) => Local::now(),
    }
}

// Formats a Datetime object into an easy to read string
pub fn format_datetime(datetime: &DateTime<Local>) -> String {
    datetime.format("%e %b %Y").to_string()
}

// Combines both datetime functions to essentially reformat a string into an easier format
pub fn reformat_datetime(text: &str) -> String {
    format_datetime(&parse_datetime(text))
}
