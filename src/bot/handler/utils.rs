use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

use crate::bot::BotError;

/* Common utilites for handlers. */

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
