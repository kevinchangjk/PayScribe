use chrono::{DateTime, Local, NaiveDateTime};
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

use crate::bot::{redis::Debt, BotError};

use super::Payment;

/* Common utilites for handlers. */

pub const MAX_VALUE: f64 = 10_000_000_000_000.00;

// Displays balances in a more readable format.
pub fn display_balances(debts: &Vec<Debt>) -> String {
    let mut message = String::new();
    for debt in debts {
        message.push_str(&format!(
            "{} owes {}: {:.2}\n",
            debt.debtor, debt.creditor, debt.amount
        ));
    }
    message
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
        "______________________________\nEntry No. {} — {}\nDate: {}\nCreditor: {}\nTotal: {:.2}\nSplit amounts:\n{}",
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

// Ensures that a username has a leading '@'.
pub fn parse_username(username: &str) -> String {
    if username.starts_with('@') {
        username.to_string()
    } else {
        format!("@{}", username)
    }
}

// Parse an amount. Reads a string, returns f64.
pub fn parse_amount(text: &str) -> Result<f64, BotError> {
    let amount = match text.parse::<f64>() {
        Ok(val) => val,
        Err(_) => match text.parse::<i32>() {
            Ok(val) => val as f64,
            Err(_) => return Err(BotError::UserError("Invalid number provided.".to_string())),
        },
    };

    if amount > MAX_VALUE {
        Err(BotError::UserError("Amount is too large.".to_string()))
    } else if amount <= 0.0 {
        Err(BotError::UserError("Amount must be positive.".to_string()))
    } else {
        Ok(amount)
    }
}

// Parse a serial number (an index). Reads a string, returns a usize.
pub fn parse_serial_num(text: &str, length: usize) -> Result<usize, BotError> {
    if text.is_empty() {
        return Err(BotError::UserError(
            "Serial number not provided!".to_string(),
        ));
    }
    let parsed_num = text.parse::<usize>();
    match parsed_num {
        Ok(serial_num) => {
            if serial_num > length || serial_num == 0 {
                Err(BotError::UserError("Invalid serial number!".to_string()))
            } else {
                Ok(serial_num)
            }
        }
        Err(_) => Err(BotError::UserError("Invalid number!".to_string())),
    }
}

// Parse and process a string to retrieve a list of debts, returns Vec<Debt>.
pub fn process_debts(
    text: &str,
    creditor: &Option<String>,
    total: Option<f64>,
) -> Result<Vec<(String, f64)>, BotError> {
    let mut debts: Vec<(String, f64)> = Vec::new();
    let mut sum: f64 = 0.0;
    let pairs: Vec<&str> = text.split(',').collect();
    for pair in pairs {
        let pair = pair.split_whitespace().collect::<Vec<&str>>();
        if pair.len() != 2 {
            return Err(BotError::UserError("Invalid format for debts.".to_string()));
        }

        let username = parse_username(pair[0]);
        let amount = parse_amount(pair[1])?;
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
                    "Total debts exceed the total amount.".to_string(),
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
                "Total amount not provided.".to_string(),
            ))
        }
    } else {
        Err(BotError::UserError("Creditor not provided.".to_string()))
    }
}

// Parses a string of debts and returns a vector of debts
pub fn parse_debts(text: &str) -> Result<Vec<(String, f64)>, BotError> {
    let mut debts: Vec<(String, f64)> = Vec::new();
    let pairs: Vec<&str> = text.split(',').collect();
    for pair in pairs {
        let pair = pair.split_whitespace().collect::<Vec<&str>>();
        if pair.len() != 2 {
            return Err(BotError::UserError("Invalid format for debts.".to_string()));
        }

        let username = parse_username(pair[0]);
        let amount = parse_amount(pair[1])?;

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
    datetime.format("%v %R").to_string()
}

// Combines both datetime functions to essentially reformat a string into an easier format
pub fn reformat_datetime(text: &str) -> String {
    format_datetime(&parse_datetime(text))
}
