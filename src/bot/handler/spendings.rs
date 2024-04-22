use teloxide::{prelude::*, types::Message};

use crate::bot::{
    currency::{Currency, CURRENCY_DEFAULT},
    handler::{
        constants::{SPENDINGS_INSTRUCTIONS_MESSAGE, UNKNOWN_ERROR_MESSAGE},
        utils::{display_amount, display_username, get_currency, HandlerResult},
    },
    processor::{
        get_chat_setting, retrieve_spending_data, ChatSetting, SpendingData, UserSpending,
    },
    State,
};

use super::utils::UserDialogue;

/* Utilities */
fn display_individual_spending(spending: UserSpending, currency: Currency) -> String {
    format!(
        "{} spent {}, and paid {}",
        display_username(&spending.username),
        display_amount(spending.spending, currency.1),
        display_amount(spending.paid, currency.1)
    )
}

fn display_spendings(spending_data: SpendingData, option: SpendingsOption) -> String {
    let currency = match get_currency(&spending_data.currency) {
        Ok(currency) => currency,
        // Should not occur. Currency string is from database, so should exist.
        Err(_) => ("NIL".to_string(), 2),
    };
    let mut individual_spendings = String::new();
    for spending in &spending_data.user_spendings {
        individual_spendings.push_str(&format!(
            "    {}\n",
            display_individual_spending(spending.clone(), currency.clone())
        ));
    }

    format!(
        "Total Group Spending: {}\nTotal Individual Spendings:\n{}",
        display_amount(spending_data.group_spending, currency.1),
        individual_spendings
    )
}

#[derive(Debug, Clone)]
pub enum SpendingsOption {
    Currency(String),
    ConvertCurrency,
}

/* View the spendings for the group.
*/
pub async fn action_view_spendings(
    bot: Bot,
    dialogue: UserDialogue,
    msg: Message,
) -> HandlerResult {
    let chat_id = msg.chat.id.to_string();
    if let Some(user) = msg.from() {
        let sender_id = user.id.to_string();
        let sender_username = user.username.clone();

        let is_convert = match get_chat_setting(&chat_id, ChatSetting::CurrencyConversion(None)) {
            Ok(ChatSetting::CurrencyConversion(Some(value))) => value,
            _ => false,
        };
        let default_currency = match get_chat_setting(&chat_id, ChatSetting::DefaultCurrency(None))
        {
            Ok(ChatSetting::DefaultCurrency(Some(currency))) => currency,
            _ => "NIL".to_string(),
        };

        let option = if is_convert {
            SpendingsOption::ConvertCurrency
        } else {
            SpendingsOption::Currency(default_currency.clone())
        };

        let spending_data = retrieve_spending_data(
            &chat_id,
            option.clone(),
            &sender_id,
            sender_username.as_deref(),
        )
        .await;

        match spending_data {
            Ok(spending_data) => {
                let header = if is_convert {
                    format!("Here are the total spendings, converted to {default_currency}!")
                } else if default_currency == CURRENCY_DEFAULT.0 {
                    format!("Here are the total spendings!")
                } else {
                    format!("Here are the total spendings for {default_currency}!")
                };

                bot.send_message(
                    chat_id,
                    format!(
                        "{}\n\n{}\n\n{SPENDINGS_INSTRUCTIONS_MESSAGE}",
                        header,
                        display_spendings(spending_data, option)
                    ),
                )
                .await?;
                dialogue.update(State::SpendingsMenu).await?;
            }
            Err(err) => {
                bot.send_message(chat_id.clone(), UNKNOWN_ERROR_MESSAGE)
                    .await?;
                log::error!(
                    "View Spendings - User {} failed to view spendings for group {}: {}",
                    sender_id,
                    chat_id,
                    err.to_string()
                );
            }
        }
    }

    Ok(())
}

/* Changes the display format of the spendings for the group.
 * Receives a callback query indicating new format change.
 */
pub async fn action_spendings_menu(
    bot: Bot,
    dialogue: UserDialogue,
    query: CallbackQuery,
) -> HandlerResult {
    Ok(())
}
