use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

use crate::bot::{redis::Debt, BotError};

/* Common utilites for handlers. */

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
            keyboard.push(vec![InlineKeyboardButton::callback(option.clone(), option)]);
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

    if amount < 0.0 {
        Err(BotError::UserError("Amount must be positive.".to_string()))
    } else {
        Ok(amount)
    }
}

// Parse a string to retrieve a list of debts, returns Vec<Debt>.
pub fn parse_debts(
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
