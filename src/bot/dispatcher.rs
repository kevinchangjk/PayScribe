use teloxide::{
    dispatching::{
        dialogue,
        dialogue::{InMemStorage, InMemStorageError},
    },
    prelude::*,
    utils::command::BotCommands,
    RequestError,
};

use crate::bot::handler::{
    action_delete_payment, action_delete_payment_confirm, action_edit_payment,
    action_edit_payment_confirm, action_edit_payment_edit, action_pay_back,
    action_pay_back_confirm, action_pay_back_debts, action_view_balances, action_view_more,
    action_view_payments, block_add_payment, block_delete_payment, block_edit_payment,
    block_pay_back, cancel_delete_payment, cancel_edit_payment, cancel_pay_back,
    handle_repeated_add_payment, handle_repeated_pay_back, no_delete_payment,
};

use super::handler::{
    action_add_confirm, action_add_creditor, action_add_debt, action_add_description,
    action_add_edit, action_add_edit_menu, action_add_payment, action_add_total, action_cancel,
    action_help, action_start, cancel_add_payment, invalid_state, AddPaymentEdit, AddPaymentParams,
    DeletePaymentParams, EditPaymentParams, PayBackParams, Payment,
};
use super::processor::ProcessError;

/* Handler is the front-facing agent of the bot.
 * It receives messages and commands from the user, and handles user interaction.
 * All user interaction, including sending and crafting of messages, is done here.
 * It communicates only with the Processor, which executes the commands.
 * User exceptions are handled in this module. Processor may propagate some errors here.dialogue
 */

/* Types */
pub type UserDialogue = Dialogue<State, InMemStorage<State>>;
pub type HandlerResult = Result<(), BotError>;

#[derive(thiserror::Error, Debug)]
pub enum BotError {
    #[error("User error: {0}")]
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

#[derive(Clone, Default)]
pub enum State {
    #[default]
    Start,
    AddDescription,
    AddCreditor {
        payment: AddPaymentParams,
    },
    AddTotal {
        payment: AddPaymentParams,
    },
    AddDebt {
        payment: AddPaymentParams,
    },
    AddOverview {
        payment: AddPaymentParams,
    },
    AddConfirm {
        payment: AddPaymentParams,
    },
    AddEditMenu {
        payment: AddPaymentParams,
    },
    AddEdit {
        payment: AddPaymentParams,
        edit: AddPaymentEdit,
    },
    PayBackDebts,
    PayBackConfirm {
        payment: PayBackParams,
    },
    ViewPayments {
        payments: Vec<Payment>,
        page: usize,
    },
    EditPayment {
        payment: EditPaymentParams,
    },
    EditPaymentDetails {
        payment: EditPaymentParams,
        edit: AddPaymentEdit,
    },
    DeletePayment {
        payment: DeletePaymentParams,
    },
}

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
pub enum Command {
    #[command(description = "Show this help message.")]
    Help,
    #[command(description = "Start the bot.")]
    Start,
    #[command(description = "Cancels an ongoing action.")]
    Cancel,
    #[command(description = "Add a payment entry for the group.")]
    AddPayment,
    #[command(description = "View the current balances for the group.")]
    ViewBalances,
    #[command(description = "View all payment records for the group.")]
    ViewPayments,
    #[command(description = "Edit a payment entry in the group.")]
    EditPayment,
    #[command(description = "Delete a payment entry in the group.")]
    DeletePayment,
    #[command(description = "Add a entry paying back other members in the group.")]
    PayBack,
}

/* Main Dispatch function */
pub async fn run_dispatcher(bot: Bot) {
    use dptree::case;

    let command_handler = teloxide::filter_command::<Command, _>()
        .branch(
            case![State::Start]
                .branch(case![Command::Start].endpoint(action_start))
                .branch(case![Command::Help].endpoint(action_help))
                .branch(case![Command::AddPayment].endpoint(action_add_payment))
                .branch(case![Command::Cancel].endpoint(action_cancel))
                .branch(case![Command::ViewBalances].endpoint(action_view_balances))
                .branch(case![Command::PayBack].endpoint(action_pay_back))
                .branch(case![Command::ViewPayments].endpoint(action_view_payments))
                .branch(case![Command::EditPayment].endpoint(block_edit_payment))
                .branch(case![Command::DeletePayment].endpoint(no_delete_payment)),
        )
        .branch(
            case![State::AddDescription]
                .branch(case![Command::Start].endpoint(action_start))
                .branch(case![Command::Help].endpoint(action_help))
                .branch(case![Command::AddPayment].endpoint(handle_repeated_add_payment))
                .branch(case![Command::Cancel].endpoint(cancel_add_payment))
                .branch(case![Command::ViewBalances].endpoint(block_add_payment))
                .branch(case![Command::PayBack].endpoint(block_add_payment))
                .branch(case![Command::ViewPayments].endpoint(block_add_payment))
                .branch(case![Command::EditPayment].endpoint(block_add_payment))
                .branch(case![Command::DeletePayment].endpoint(block_add_payment)),
        )
        .branch(
            case![State::AddCreditor { payment }]
                .branch(case![Command::Start].endpoint(action_start))
                .branch(case![Command::Help].endpoint(action_help))
                .branch(case![Command::AddPayment].endpoint(handle_repeated_add_payment))
                .branch(case![Command::Cancel].endpoint(cancel_add_payment))
                .branch(case![Command::ViewBalances].endpoint(block_add_payment))
                .branch(case![Command::PayBack].endpoint(block_add_payment))
                .branch(case![Command::ViewPayments].endpoint(block_add_payment))
                .branch(case![Command::EditPayment].endpoint(block_add_payment))
                .branch(case![Command::DeletePayment].endpoint(block_add_payment)),
        )
        .branch(
            case![State::AddTotal { payment }]
                .branch(case![Command::Start].endpoint(action_start))
                .branch(case![Command::Help].endpoint(action_help))
                .branch(case![Command::AddPayment].endpoint(handle_repeated_add_payment))
                .branch(case![Command::Cancel].endpoint(cancel_add_payment))
                .branch(case![Command::ViewBalances].endpoint(block_add_payment))
                .branch(case![Command::PayBack].endpoint(block_add_payment))
                .branch(case![Command::ViewPayments].endpoint(block_add_payment))
                .branch(case![Command::EditPayment].endpoint(block_add_payment))
                .branch(case![Command::DeletePayment].endpoint(block_add_payment)),
        )
        .branch(
            case![State::AddDebt { payment }]
                .branch(case![Command::Start].endpoint(action_start))
                .branch(case![Command::Help].endpoint(action_help))
                .branch(case![Command::AddPayment].endpoint(handle_repeated_add_payment))
                .branch(case![Command::Cancel].endpoint(cancel_add_payment))
                .branch(case![Command::ViewBalances].endpoint(block_add_payment))
                .branch(case![Command::PayBack].endpoint(block_add_payment))
                .branch(case![Command::ViewPayments].endpoint(block_add_payment))
                .branch(case![Command::EditPayment].endpoint(block_add_payment))
                .branch(case![Command::DeletePayment].endpoint(block_add_payment)),
        )
        .branch(
            case![State::AddConfirm { payment }]
                .branch(case![Command::Start].endpoint(action_start))
                .branch(case![Command::Help].endpoint(action_help))
                .branch(case![Command::AddPayment].endpoint(handle_repeated_add_payment))
                .branch(case![Command::Cancel].endpoint(cancel_add_payment))
                .branch(case![Command::ViewBalances].endpoint(block_add_payment))
                .branch(case![Command::PayBack].endpoint(block_add_payment))
                .branch(case![Command::ViewPayments].endpoint(block_add_payment))
                .branch(case![Command::EditPayment].endpoint(block_add_payment))
                .branch(case![Command::DeletePayment].endpoint(block_add_payment)),
        )
        .branch(
            case![State::AddEditMenu { payment }]
                .branch(case![Command::Start].endpoint(action_start))
                .branch(case![Command::Help].endpoint(action_help))
                .branch(case![Command::AddPayment].endpoint(handle_repeated_add_payment))
                .branch(case![Command::Cancel].endpoint(cancel_add_payment))
                .branch(case![Command::ViewBalances].endpoint(block_add_payment))
                .branch(case![Command::PayBack].endpoint(block_add_payment))
                .branch(case![Command::ViewPayments].endpoint(block_add_payment))
                .branch(case![Command::EditPayment].endpoint(block_add_payment))
                .branch(case![Command::DeletePayment].endpoint(block_add_payment)),
        )
        .branch(
            case![State::AddEdit { payment, edit }]
                .branch(case![Command::Start].endpoint(action_start))
                .branch(case![Command::Help].endpoint(action_help))
                .branch(case![Command::AddPayment].endpoint(handle_repeated_add_payment))
                .branch(case![Command::Cancel].endpoint(cancel_add_payment))
                .branch(case![Command::ViewBalances].endpoint(block_add_payment))
                .branch(case![Command::PayBack].endpoint(block_add_payment))
                .branch(case![Command::ViewPayments].endpoint(block_add_payment))
                .branch(case![Command::EditPayment].endpoint(block_add_payment))
                .branch(case![Command::DeletePayment].endpoint(block_add_payment)),
        )
        .branch(
            case![State::PayBackDebts]
                .branch(case![Command::Start].endpoint(action_start))
                .branch(case![Command::Help].endpoint(action_help))
                .branch(case![Command::AddPayment].endpoint(block_pay_back))
                .branch(case![Command::Cancel].endpoint(cancel_pay_back))
                .branch(case![Command::ViewBalances].endpoint(block_pay_back))
                .branch(case![Command::PayBack].endpoint(handle_repeated_pay_back))
                .branch(case![Command::ViewPayments].endpoint(block_pay_back))
                .branch(case![Command::EditPayment].endpoint(block_pay_back))
                .branch(case![Command::DeletePayment].endpoint(block_pay_back)),
        )
        .branch(
            case![State::PayBackConfirm { payment }]
                .branch(case![Command::Start].endpoint(action_start))
                .branch(case![Command::Help].endpoint(action_help))
                .branch(case![Command::AddPayment].endpoint(block_pay_back))
                .branch(case![Command::Cancel].endpoint(cancel_pay_back))
                .branch(case![Command::ViewBalances].endpoint(block_pay_back))
                .branch(case![Command::PayBack].endpoint(handle_repeated_pay_back))
                .branch(case![Command::ViewPayments].endpoint(block_pay_back))
                .branch(case![Command::EditPayment].endpoint(block_pay_back))
                .branch(case![Command::DeletePayment].endpoint(block_pay_back)),
        )
        .branch(
            case![State::ViewPayments { payments, page }]
                .branch(case![Command::Start].endpoint(action_start))
                .branch(case![Command::Help].endpoint(action_help))
                .branch(case![Command::AddPayment].endpoint(action_add_payment))
                .branch(case![Command::Cancel].endpoint(action_cancel))
                .branch(case![Command::ViewBalances].endpoint(action_view_payments))
                .branch(case![Command::PayBack].endpoint(action_pay_back))
                .branch(case![Command::ViewPayments].endpoint(action_view_payments))
                .branch(case![Command::EditPayment].endpoint(action_edit_payment))
                .branch(case![Command::DeletePayment].endpoint(action_delete_payment)),
        )
        .branch(
            case![State::EditPayment { payment }]
                .branch(case![Command::Start].endpoint(action_start))
                .branch(case![Command::Help].endpoint(action_help))
                .branch(case![Command::AddPayment].endpoint(block_edit_payment))
                .branch(case![Command::Cancel].endpoint(cancel_edit_payment))
                .branch(case![Command::ViewBalances].endpoint(block_edit_payment))
                .branch(case![Command::PayBack].endpoint(block_edit_payment))
                .branch(case![Command::ViewPayments].endpoint(block_edit_payment))
                .branch(case![Command::EditPayment].endpoint(action_edit_payment))
                .branch(case![Command::DeletePayment].endpoint(block_edit_payment)),
        )
        .branch(
            case![State::EditPaymentDetails { payment, edit }]
                .branch(case![Command::Start].endpoint(action_start))
                .branch(case![Command::Help].endpoint(action_help))
                .branch(case![Command::AddPayment].endpoint(block_edit_payment))
                .branch(case![Command::Cancel].endpoint(cancel_edit_payment))
                .branch(case![Command::ViewBalances].endpoint(block_edit_payment))
                .branch(case![Command::PayBack].endpoint(block_edit_payment))
                .branch(case![Command::ViewPayments].endpoint(block_edit_payment))
                .branch(case![Command::EditPayment].endpoint(action_edit_payment))
                .branch(case![Command::DeletePayment].endpoint(block_edit_payment)),
        )
        .branch(
            case![State::DeletePayment { payment }]
                .branch(case![Command::Start].endpoint(action_start))
                .branch(case![Command::Help].endpoint(action_help))
                .branch(case![Command::AddPayment].endpoint(block_delete_payment))
                .branch(case![Command::Cancel].endpoint(cancel_delete_payment))
                .branch(case![Command::ViewBalances].endpoint(block_delete_payment))
                .branch(case![Command::PayBack].endpoint(block_delete_payment))
                .branch(case![Command::ViewPayments].endpoint(block_delete_payment))
                .branch(case![Command::EditPayment].endpoint(block_delete_payment))
                .branch(case![Command::DeletePayment].endpoint(action_delete_payment)),
        );

    let message_handler = Update::filter_message()
        .branch(command_handler)
        .branch(case![State::AddDescription].endpoint(action_add_description))
        .branch(case![State::AddCreditor { payment }].endpoint(action_add_creditor))
        .branch(case![State::AddTotal { payment }].endpoint(action_add_total))
        .branch(case![State::AddDebt { payment }].endpoint(action_add_debt))
        .branch(case![State::AddEdit { payment, edit }].endpoint(action_add_edit))
        .branch(case![State::PayBackDebts].endpoint(action_pay_back_debts))
        .branch(
            case![State::EditPaymentDetails { payment, edit }].endpoint(action_edit_payment_edit),
        )
        .branch(dptree::endpoint(invalid_state));

    let callback_query_handler = Update::filter_callback_query()
        .branch(case![State::AddConfirm { payment }].endpoint(action_add_confirm))
        .branch(case![State::AddEditMenu { payment }].endpoint(action_add_edit_menu))
        .branch(case![State::PayBackConfirm { payment }].endpoint(action_pay_back_confirm))
        .branch(case![State::ViewPayments { payments, page }].endpoint(action_view_more))
        .branch(case![State::EditPayment { payment }].endpoint(action_edit_payment_confirm))
        .branch(case![State::DeletePayment { payment }].endpoint(action_delete_payment_confirm));

    let schema = dialogue::enter::<Update, InMemStorage<State>, State, _>()
        .branch(message_handler)
        .branch(callback_query_handler);

    Dispatcher::builder(bot, schema)
        .dependencies(dptree::deps![InMemStorage::<State>::new()])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}
