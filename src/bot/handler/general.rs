use teloxide::{prelude::*, types::ParseMode, utils::command::BotCommands};

use crate::bot::dispatcher::Command;

use super::utils::{
    HandlerResult, COMMAND_ADD_PAYMENT, COMMAND_DELETE_PAYMENT, COMMAND_EDIT_PAYMENT, COMMAND_HELP,
    COMMAND_PAY_BACK, COMMAND_VIEW_BALANCES, COMMAND_VIEW_PAYMENTS,
};

/* Invalid state.
*/
pub async fn invalid_state(bot: Bot, msg: Message) -> HandlerResult {
    // Checks if msg is a service message, ignores it if so
    let is_service_msg = msg.from().is_none();

    if is_service_msg {
        Ok(())
    } else {
        bot.send_message(msg.chat.id, format!("Sorry, I'm not intelligent enough to process that! 🤖\nPlease refer to {COMMAND_HELP} on how to use me!")).await?;
        Ok(())
    }
}

/* Invalid message during callback expected.
*/
pub async fn callback_invalid_message(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(
        msg.chat.id,
        "Hey, you don't have to text me...🗿\nJust click on any of the buttons above 👆 to continue!",
        )
        .await?;
    Ok(())
}

/* Start command.
 * Displays a welcome message to the user.
 */
pub async fn action_start(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, format!("👋 Hi, I'm PayScribe!\n\nEnter {COMMAND_HELP} to check out the various commands I can assist you with, and let's get straight into tracking payments together!")).await?;
    Ok(())
}

/* Help command.
 * Displays a list of commands available to the user.
 */
pub async fn action_help(bot: Bot, msg: Message) -> HandlerResult {
    let mut commands = Command::descriptions().to_string();
    commands = commands.replace("–", "\\—");

    let introduction = "👋 Hey there\\! Need some help?\n\n_PayScribe_ is a handy assistant for keeping track of group payments, and simplifying your debts to keep you updated with how much everyone owes one another\\.";
    let add_info = &format!("To begin, you can add new payment records with {COMMAND_ADD_PAYMENT}\\. When splitting the total amount, you can:\n\\- Divide the total cost equally \\(e\\.g\\. sharing ticket prices equally among friends\\)\n\\- Specify the exact amount for each person\n\\- Provide a proportion of how much each person owes \\(e\\.g\\. splitting the electricity bill 40\\-60\\)");
    let view_info = &format!("Use {COMMAND_VIEW_BALANCES} see how much everyone owes one another\\. To edit or delete payments, use {COMMAND_VIEW_PAYMENTS}, then {COMMAND_EDIT_PAYMENT} or {COMMAND_DELETE_PAYMENT}\\.");
    let payback_info = &format!("After paying back your friends, be sure to record those down with the {COMMAND_PAY_BACK} command\\!");
    let access_info = "You will have to directly reply to my messages for me to read your texts\\. If I'm being too nosy and responding to every message, you might have accidentally given me admin rights\\!";

    bot.send_message(
        msg.chat.id,
        format!("{introduction}\n\n{add_info}\n\n{view_info}\n\n{payback_info}\n\n{access_info}\n\n*Check out all my commands here*:\n\n{}", commands),
        )
        .parse_mode(ParseMode::MarkdownV2)
        .await?;
    Ok(())
}

/* Cancel command.
 * Called when state is at start, thus nothing to cancel.
 */
pub async fn action_cancel(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(
        msg.chat.id,
        "I'm not doing anything right now. There's nothing to cancel! 👀",
    )
    .await?;
    Ok(())
}

/* Currencies command.
 * Displays a list of all supported currencies
 */
pub async fn action_currencies(bot: Bot, msg: Message) -> HandlerResult {
    let currencies = vec![
        "Australian Dollar — *AUD*",
        "Canadian Dollar — *CAD*",
        "Chinese Yuan — *CNY*",
        "Euro — *EUR*",
        "Great Britain Pound — *GBP*",
        "Hong Kong Dollar — *HKD*",
        "Indian Rupee — *INR*",
        "Indonesian Rupiah — *IDR*",
        "Japanese Yen — *JPY*",
        "Malaysian Ringgit — *MYR*",
        "Mexican Peso — *MXN*",
        "New Zealand Dollar — *NZD*",
        "Philippine Peso — *PHP*",
        "Russian Ruble — *RUB*",
        "Saudi Riyal — *SAR*",
        "Singapore Dollar — *SGD*",
        "South Korean Won — *KRW*",
        "Swedish Krona — *SEK*",
        "Swiss Franc — *CHF*",
        "Taiwan Dollar — *TWD*",
        "Thai Baht — *THB*",
        "Turkish Lira — *TRY*",
        "UAE Dirham — *AED*",
        "United States Dollar — *USD*",
        "Vietnamese Dong — *VND*",
    ];
    bot.send_message(
        msg.chat.id,
        format!(
            "Here are the currencies I know\\!\n\n{}",
            currencies.join("\n")
        ),
    )
    .parse_mode(ParseMode::MarkdownV2)
    .await?;
    Ok(())
}
