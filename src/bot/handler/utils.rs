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

pub const MAX_VALUE: i64 = 1_000_000_000_000_000_000;
pub const UNKNOWN_ERROR_MESSAGE: &str =
    "❓ Hmm, something went wrong! Sorry, I can't do that right now, please try again later!\n\n";
pub const NO_TEXT_MESSAGE: &str =
    "❓ Sorry, I can't understand that! Please reply to me in text.\n\n";
pub const DEBT_EQUAL_DESCRIPTION_MESSAGE: &str =
    "Equal — Divide the total amount equally among users\n";
pub const DEBT_EXACT_DESCRIPTION_MESSAGE: &str =
    "Exact — Share the total cost by specifying exact amounts for each user\n";
pub const DEBT_RATIO_DESCRIPTION_MESSAGE: &str =
    "Proportion — Split the total cost by specifying relative proportions of the total that each user owes\n";
pub const DEBT_EQUAL_INSTRUCTIONS_MESSAGE: &str =
    "Enter the usernames of those sharing the cost (including the payer if sharing too) as follows: \n\n@username__1\n@username__2\n@username__3\n...\n\n";
pub const DEBT_EXACT_INSTRUCTIONS_MESSAGE: &str =
    "Enter the usernames and exact amounts as follows: \n\n@username__1 amount1\n@username__2 amount2\n@username__3 amount3\n...\n\nAny leftover amount will be taken as the payer's share.";
pub const PAY_BACK_INSTRUCTIONS_MESSAGE: &str =
    "Enter the usernames and exact amounts as follows: \n\n@username__1 amount1\n@username__2 amount2\n@username__3 amount3\n...\n\n";
pub const DEBT_RATIO_INSTRUCTIONS_MESSAGE: &str =
    "Enter the usernames and proportions as follows: \n\n@username__1 portion1\n@username__2 portion2\n@username__3 portion3\n...\n\nThe portions can be any whole or decimal number.";
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
// Represents a currency with a code and decimal places.
pub type Currency = (String, i32);

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

// List of all supported currencies
pub const CURRENCIES: [Currency; 4] = [
    ("JPY".to_string(), 0),
    ("USD".to_string(), 2),
    ("SGD".to_string(), 2),
    ("MYR".to_string(), 2),
];

pub const CURRENCY_DEFAULT: Currency = ("NIL".to_string(), 2);

// Retrieves the currency given a currency code.
pub fn get_currency(code: &str) -> Result<Currency, BotError> {
    for currency in &CURRENCIES {
        if currency.0 == code {
            return Ok(currency.clone());
        }
    }

    Err(BotError::UserError(
        "❌ Sorry, I don't have that currency!".to_string(),
    ))
}

// Converts an amount from base value to actual representation in currency.
pub fn display_amount(amount: i64, decimal_places: i32) -> String {
    if decimal_places == 0 {
        return amount.to_string();
    } else if amount == 0 {
        return "0".to_string();
    }

    // Amount is not 0, and decimal places are not 0
    let factor = 10.0_f64.powi(decimal_places);
    let actual_amount = amount as f64 / factor;
    format!(
        "{:.decimals$}",
        actual_amount,
        decimals = decimal_places as usize
    )
}

// Displays an amount together with its currency
pub fn display_currency_amount(amount: i64, currency: Currency) -> String {
    format!("{} {}", display_amount(amount, currency.1), currency.0)
}

// Displays balances in a more readable format.
pub fn display_balances(debts: &Vec<Debt>) -> String {
    let mut message = String::new();
    let currency = ("SGD".to_string(), 2); // TODO
    for debt in debts {
        message.push_str(&format!(
            "{} owes {}: {}\n",
            display_username(&debt.debtor),
            display_username(&debt.creditor),
            display_currency_amount(debt.amount, currency),
        ));
    }

    if message.is_empty() {
        "No outstanding balances! 👍".to_string()
    } else {
        message
    }
}

// Displays debts in a more readable format.
pub fn display_debts(debts: &Vec<(String, i64)>, decimal_places: i32) -> String {
    let mut message = String::new();
    for debt in debts {
        message.push_str(&format!(
            "    {}: {}\n",
            display_username(&debt.0),
            display_amount(debt.1, decimal_places),
        ));
    }
    message
}

// Displays a single payment entry in a user-friendly format.
pub fn display_payment(payment: &Payment, serial_num: usize) -> String {
    format!(
        "__________________________\n{}. {}\nDate: {}\nPayer: {}\nTotal: {}\nSplit:\n{}",
        serial_num,
        payment.description,
        reformat_datetime(&payment.datetime),
        display_username(&payment.creditor),
        display_currency_amount(payment.total, payment.currency),
        display_debts(&payment.debts, payment.currency.1)
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
    let buttons = vec!["Equal", "Exact", "Proportion"];
    make_keyboard(buttons, Some(3))
}

// Displays a username with the '@' symbol.
pub fn display_username(username: &str) -> String {
    format!("@{}", username)
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
                return Ok(text.to_string());
            }
        }
    }

    Err(BotError::UserError(
        "❌ Please provide a valid username!".to_string(),
    ))
}

// Parse an amount. Reads a string, returns i64 based on currency.
pub fn parse_amount(text: &str, decimal_places: i32) -> Result<i64, BotError> {
    let amount = match text.parse::<i64>() {
        Ok(val) => val,
        Err(_) => match text.parse::<f64>() {
            Ok(val) => {
                let factor = 10.0_f64.powi(decimal_places);
                (val * factor).round() as i64
            }
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
    } else if amount <= 0 {
        Err(BotError::UserError(
            "❌ Please provide a positive value!".to_string(),
        ))
    } else {
        Ok(amount)
    }
}

// Parse a float. Reads a string, returns f64.
pub fn parse_float(text: &str) -> Result<f64, BotError> {
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

    if amount > MAX_VALUE as f64 {
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
pub fn process_debts_equal(text: &str, total: Option<i64>) -> Result<Vec<(String, i64)>, BotError> {
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

    let amount = (total as f64 / users.len() as f64).round() as i64;
    let diff = total - amount * users.len() as i64;
    let mut debts: Vec<(String, i64)> = Vec::new();
    for user in &users {
        let debt = (parse_username(user)?, amount);
        debts.push(debt);
    }

    // Distribute the difference in amount to the first user (<= smallest denomination)
    debts[0].1 += diff;

    Ok(debts)
}

// Parse and process a string to retrieve a list of debts, for split by exact amount.
pub fn process_debts_exact(
    text: &str,
    creditor: &Option<String>,
    currency: Currency,
    total: Option<i64>,
) -> Result<Vec<(String, i64)>, BotError> {
    let mut debts: Vec<(String, i64)> = Vec::new();
    let mut sum: i64 = 0;
    let items: Vec<&str> = text.split_whitespace().collect();
    if items.len() % 2 != 0 {
        return Err(BotError::UserError(
            "❌ Please use the following format!".to_string(),
        ));
    }

    for i in (0..items.len()).step_by(2) {
        let username = parse_username(items[i])?;
        let amount = parse_amount(items[i + 1], currency.1)?;
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
pub fn process_debts_ratio(
    text: &str,
    currency: Currency,
    total: Option<i64>,
) -> Result<Vec<(String, i64)>, BotError> {
    let items: Vec<&str> = text.split_whitespace().collect();
    let mut debts_ratioed: Vec<(String, f64)> = Vec::new();
    let mut debts: Vec<(String, i64)> = Vec::new();
    let mut sum: f64 = 0.0;

    if items.len() % 2 != 0 {
        return Err(BotError::UserError(
            "❌ Please use the following format!".to_string(),
        ));
    }

    for i in (0..items.len()).step_by(2) {
        let username = parse_username(items[i])?;
        let ratio = parse_float(items[i + 1])?;

        sum += ratio;
        debts_ratioed.push((username, ratio));
    }

    let total = match total {
        Some(val) => val,
        None => {
            return Err(BotError::UserError(
                "❌ Something's wrong! The total amount isn't provided.".to_string(),
            ));
        }
    };

    let mut exact_sum: i64 = 0;
    for debt in &mut debts_ratioed {
        let amount = ((debt.1 / sum) * total as f64).round() as i64;
        debts.push((debt.0, amount));
        exact_sum += amount;
    }

    // Distribute the difference in amount to the first user (<= smallest denomination)
    let diff = total - exact_sum;
    debts[0].1 += diff;

    Ok(debts)
}

// Parse and process a string to retrieve a list of debts, returns Vec<Debt>.
pub fn process_debts(
    debts_format: AddDebtsFormat,
    text: &str,
    creditor: &Option<String>,
    currency: Currency,
    total: Option<i64>,
) -> Result<Vec<(String, i64)>, BotError> {
    match debts_format {
        AddDebtsFormat::Equal => process_debts_equal(text, total),
        AddDebtsFormat::Exact => process_debts_exact(text, creditor, currency, total),
        AddDebtsFormat::Ratio => process_debts_ratio(text, currency, total),
    }
}

// Parses a string of debts and returns a vector of debts
pub fn parse_debts_payback(
    text: &str,
    currency: Currency,
    sender: &str,
) -> Result<Vec<(String, i64)>, BotError> {
    let mut debts: Vec<(String, i64)> = Vec::new();
    let items: Vec<&str> = text.split_whitespace().collect();
    if items.len() % 2 != 0 {
        return Err(BotError::UserError(
            "❌ Please use the following format!".to_string(),
        ));
    }

    for i in (0..items.len()).step_by(2) {
        let username = parse_username(items[i])?;
        let amount = parse_amount(items[i + 1], currency.1)?;
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
