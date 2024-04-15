use chrono::{DateTime, Local, NaiveDateTime};
use chrono_tz::Tz;
use regex::Regex;
use teloxide::{
    dispatching::dialogue::{Dialogue, InMemStorage, InMemStorageError},
    types::{InlineKeyboardButton, InlineKeyboardMarkup},
    RequestError,
};

use crate::bot::{
    currency::{get_currency_from_code, get_default_currency, CURRENCY_DEFAULT},
    processor::{get_chat_setting, ChatSetting, ProcessError},
    redis::Debt,
    State,
};

use super::{
    constants::{all_time_zones, MAX_VALUE},
    AddDebtsFormat, Payment,
};

/* Common utilites for handlers. */

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

// Retrieves the currency given a currency code.
pub fn get_currency(code: &str) -> Result<Currency, BotError> {
    let currency = get_currency_from_code(code);
    match currency {
        Some(currency) => Ok(currency),
        None => Err(BotError::UserError(
            "❌ Sorry, I don't have that currency!".to_string(),
        )),
    }
}

// Retrieves the default currency of a chat. Does not return an error, assumes default.
pub fn get_chat_default_currency(chat_id: &str) -> Currency {
    let setting = ChatSetting::DefaultCurrency(None);
    let currency = get_chat_setting(&chat_id, setting);
    match currency {
        Ok(ChatSetting::DefaultCurrency(Some(currency))) => {
            let currency = get_currency(&currency);
            if let Ok(currency) = currency {
                return currency;
            }
        }
        // Skips error, assumes default
        _ => {}
    }
    get_default_currency()
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
    if currency.0 == CURRENCY_DEFAULT.0 {
        format!("{}", display_amount(amount, currency.1))
    } else {
        format!("{} {}", display_amount(amount, currency.1), currency.0)
    }
}

// Gets the currency to be used when provided with the chosen currency, and the chat ID.
pub fn use_currency(currency: Currency, chat_id: &str) -> Currency {
    let default_currency = get_chat_default_currency(chat_id);
    if currency.0 == CURRENCY_DEFAULT.0 {
        default_currency
    } else {
        currency
    }
}

// Displays balances in a more readable format.
pub fn display_balances(debts: &Vec<Debt>, chat_id: &str) -> String {
    let mut message = String::new();
    for debt in debts {
        let currency = get_currency(&debt.currency);
        match currency {
            Ok(currency) => {
                let actual_currency = use_currency(currency, chat_id);
                message.push_str(&format!(
                    "{} owes {}: {}\n",
                    display_username(&debt.debtor),
                    display_username(&debt.creditor),
                    display_currency_amount(debt.amount, actual_currency),
                ));
            }
            // Should not occur, since code is already processed and stored in database
            Err(_err) => {
                continue;
            }
        }
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
pub fn display_payment(payment: &Payment, serial_num: usize, time_zone: Tz) -> String {
    let actual_currency = use_currency(payment.currency.clone(), &payment.chat_id);

    format!(
        "__________________________\n{}. {}\nDate: {}\nPayer: {}\nTotal: {}\nSplit:\n{}",
        serial_num,
        payment.description,
        reformat_datetime(&payment.datetime, time_zone),
        display_username(&payment.creditor),
        display_currency_amount(payment.total, actual_currency.clone()),
        display_debts(&payment.debts, actual_currency.1)
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
    let factor = 10.0_f64.powi(decimal_places);
    let amount = match text.parse::<i64>() {
        Ok(val) => (val as f64 * factor).round() as i64,
        Err(_) => match text.parse::<f64>() {
            Ok(val) => (val * factor).round() as i64,
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

// Parse a string representing an amount and a currency
pub fn parse_currency_amount(text: &str) -> Result<(i64, Currency), BotError> {
    let items = text.split_whitespace().collect::<Vec<&str>>();
    if items.len() > 2 {
        return Err(BotError::UserError(
            "❌ Please use the following format!".to_string(),
        ));
    } else if items.len() == 1 {
        let currency = get_default_currency();
        let amount = parse_amount(items[0], currency.1)?;
        Ok((amount, currency))
    } else {
        let currency = get_currency(&items[1])?;
        let amount = parse_amount(items[0], currency.1)?;
        Ok((amount, currency))
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
    currency: Option<Currency>,
    total: Option<i64>,
) -> Result<Vec<(String, i64)>, BotError> {
    if let Some(creditor) = creditor {
        if let Some(total) = total {
            if let Some(currency) = currency {
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
                    "❌ Something's wrong! The currency isn't provided.".to_string(),
                ))
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
pub fn process_debts_ratio(text: &str, total: Option<i64>) -> Result<Vec<(String, i64)>, BotError> {
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
        debts.push((debt.0.clone(), amount));
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
    currency: Option<Currency>,
    total: Option<i64>,
) -> Result<Vec<(String, i64)>, BotError> {
    match debts_format {
        AddDebtsFormat::Equal => process_debts_equal(text, total),
        AddDebtsFormat::Exact => process_debts_exact(text, creditor, currency, total),
        AddDebtsFormat::Ratio => process_debts_ratio(text, total),
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

// Parses a string representing a time zone, and returns the TimeZone object
pub fn parse_time_zone(text: &str) -> Result<Tz, BotError> {
    let text = text.to_lowercase();
    let time_zones = all_time_zones();
    let time_zone = time_zones.get(&text);

    match time_zone {
        Some(time_zone) => Ok(*time_zone),
        None => Err(BotError::UserError(
            "❌ Sorry, I don't recognize that time zone!".to_string(),
        )),
    }
}

// Retrieves the time zone string from database, converts it to TimeZone object
// Assumes that time zone is valid, thus does not return any error
pub fn retrieve_time_zone(chat_id: &str) -> Tz {
    let setting = ChatSetting::TimeZone(None);
    let time_zone = get_chat_setting(&chat_id, setting);
    if let Ok(ChatSetting::TimeZone(Some(time_zone))) = time_zone {
        let time_zone = parse_time_zone(&time_zone);
        if let Ok(time_zone) = time_zone {
            return time_zone;
        }
    }

    "UTC".parse::<Tz>().expect("UTC is a valid time zone")
}

// Parses a string representing a datetime, and returns the Datetime object
fn parse_datetime(text: &str, time_zone: Tz) -> DateTime<Tz> {
    // Checks if text contains "UTC" at the end
    let mut new_text = text.to_string();
    if text.ends_with(" UTC") {
        new_text = new_text.replace(" UTC", "");
    }
    let datetime = NaiveDateTime::parse_from_str(&new_text, "%Y-%m-%d %H:%M:%S");
    match datetime {
        Ok(val) => val.and_utc().with_timezone(&time_zone),
        Err(_) => Local::now().with_timezone(&time_zone),
    }
}

// Formats a Datetime object into an easy to read string
fn format_datetime(datetime: &DateTime<Tz>) -> String {
    datetime.format("%e %b %Y %T").to_string()
}

// Combines both datetime functions to essentially reformat a string into an easier format
fn reformat_datetime(text: &str, time_zone: Tz) -> String {
    format_datetime(&parse_datetime(text, time_zone))
}
