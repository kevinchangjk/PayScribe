use teloxide::{
    dispatching::{dialogue, dialogue::InMemStorage},
    prelude::*,
    utils::command::BotCommands,
};

use crate::bot::handler::{
    action_delete_payment, action_delete_payment_confirm, action_edit_payment,
    action_edit_payment_confirm, action_edit_payment_edit, action_pay_back,
    action_pay_back_confirm, action_pay_back_debts, action_view_balances, action_view_more,
    action_view_payments, block_add_payment, block_delete_payment, block_edit_payment,
    block_pay_back, callback_invalid_message, cancel_delete_payment, cancel_edit_payment,
    cancel_pay_back, handle_repeated_add_payment, handle_repeated_delete_payment,
    handle_repeated_edit_payment, handle_repeated_pay_back, no_delete_payment, no_edit_payment,
};

use super::handler::{
    action_add_confirm, action_add_creditor, action_add_debt, action_add_description,
    action_add_edit, action_add_edit_menu, action_add_payment, action_add_total, action_cancel,
    action_help, action_start, cancel_add_payment, invalid_state, AddPaymentEdit, AddPaymentParams,
    EditPaymentParams, PayBackParams, Payment,
};

/* Handler is the front-facing agent of the bot.
 * It receives messages and commands from the user, and handles user interaction.
 * All user interaction, including sending and crafting of messages, is done here.
 * It communicates only with the Processor, which executes the commands.
 * User exceptions are handled in this module. Processor may propagate some errors here.dialogue
 */

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
        payment: Payment,
        edited_payment: EditPaymentParams,
        payments: Vec<Payment>,
        page: usize,
    },
    EditPaymentDetails {
        payment: Payment,
        edited_payment: EditPaymentParams,
        edit: AddPaymentEdit,
        payments: Vec<Payment>,
        page: usize,
    },
    DeletePayment {
        payment: Payment,
        payments: Vec<Payment>,
        page: usize,
    },
}

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
pub enum Command {
    #[command(description = "Starts the bot")]
    Start,
    #[command(description = "Show all commands, and how to use the bot")]
    Help,
    #[command(description = "Add a payment entry for the group")]
    AddPayment,
    #[command(description = "Add a entry paying back other members in the group")]
    PayBack,
    #[command(description = "View all payment records for the group")]
    ViewPayments,
    #[command(description = "Edit a payment record previously added")]
    EditPayment { serial_num: String },
    #[command(description = "Delete a payment record previously added")]
    DeletePayment { serial_num: String },
    #[command(description = "View the current balances for the group")]
    ViewBalances,
    #[command(description = "Cancels an ongoing action")]
    Cancel,
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
                .branch(case![Command::EditPayment { serial_num }].endpoint(no_edit_payment))
                .branch(case![Command::DeletePayment { serial_num }].endpoint(no_delete_payment)),
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
                .branch(case![Command::EditPayment { serial_num }].endpoint(block_add_payment))
                .branch(case![Command::DeletePayment { serial_num }].endpoint(block_add_payment)),
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
                .branch(case![Command::EditPayment { serial_num }].endpoint(block_add_payment))
                .branch(case![Command::DeletePayment { serial_num }].endpoint(block_add_payment)),
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
                .branch(case![Command::EditPayment { serial_num }].endpoint(block_add_payment))
                .branch(case![Command::DeletePayment { serial_num }].endpoint(block_add_payment)),
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
                .branch(case![Command::EditPayment { serial_num }].endpoint(block_add_payment))
                .branch(case![Command::DeletePayment { serial_num }].endpoint(block_add_payment)),
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
                .branch(case![Command::EditPayment { serial_num }].endpoint(block_add_payment))
                .branch(case![Command::DeletePayment { serial_num }].endpoint(block_add_payment)),
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
                .branch(case![Command::EditPayment { serial_num }].endpoint(block_add_payment))
                .branch(case![Command::DeletePayment { serial_num }].endpoint(block_add_payment)),
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
                .branch(case![Command::EditPayment { serial_num }].endpoint(block_add_payment))
                .branch(case![Command::DeletePayment { serial_num }].endpoint(block_add_payment)),
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
                .branch(case![Command::EditPayment { serial_num }].endpoint(block_pay_back))
                .branch(case![Command::DeletePayment { serial_num }].endpoint(block_pay_back)),
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
                .branch(case![Command::EditPayment { serial_num }].endpoint(block_pay_back))
                .branch(case![Command::DeletePayment { serial_num }].endpoint(block_pay_back)),
        )
        .branch(
            case![State::ViewPayments { payments, page }]
                .branch(case![Command::Start].endpoint(action_start))
                .branch(case![Command::Help].endpoint(action_help))
                .branch(case![Command::AddPayment].endpoint(action_add_payment))
                .branch(case![Command::Cancel].endpoint(action_cancel))
                .branch(case![Command::ViewBalances].endpoint(action_view_balances))
                .branch(case![Command::PayBack].endpoint(action_pay_back))
                .branch(case![Command::ViewPayments].endpoint(action_view_payments))
                .branch(case![Command::EditPayment { serial_num }].endpoint(action_edit_payment))
                .branch(
                    case![Command::DeletePayment { serial_num }].endpoint(action_delete_payment),
                ),
        )
        .branch(
            case![State::EditPayment {
                payment,
                edited_payment,
                payments,
                page
            }]
            .branch(case![Command::Start].endpoint(action_start))
            .branch(case![Command::Help].endpoint(action_help))
            .branch(case![Command::AddPayment].endpoint(block_edit_payment))
            .branch(case![Command::Cancel].endpoint(cancel_edit_payment))
            .branch(case![Command::ViewBalances].endpoint(block_edit_payment))
            .branch(case![Command::PayBack].endpoint(block_edit_payment))
            .branch(case![Command::ViewPayments].endpoint(block_edit_payment))
            .branch(
                case![Command::EditPayment { serial_num }].endpoint(handle_repeated_edit_payment),
            )
            .branch(case![Command::DeletePayment { serial_num }].endpoint(block_edit_payment)),
        )
        .branch(
            case![State::EditPaymentDetails {
                payment,
                edited_payment,
                edit,
                payments,
                page
            }]
            .branch(case![Command::Start].endpoint(action_start))
            .branch(case![Command::Help].endpoint(action_help))
            .branch(case![Command::AddPayment].endpoint(block_edit_payment))
            .branch(case![Command::Cancel].endpoint(cancel_edit_payment))
            .branch(case![Command::ViewBalances].endpoint(block_edit_payment))
            .branch(case![Command::PayBack].endpoint(block_edit_payment))
            .branch(case![Command::ViewPayments].endpoint(block_edit_payment))
            .branch(
                case![Command::EditPayment { serial_num }].endpoint(handle_repeated_edit_payment),
            )
            .branch(case![Command::DeletePayment { serial_num }].endpoint(block_edit_payment)),
        )
        .branch(
            case![State::DeletePayment {
                payment,
                payments,
                page
            }]
            .branch(case![Command::Start].endpoint(action_start))
            .branch(case![Command::Help].endpoint(action_help))
            .branch(case![Command::AddPayment].endpoint(block_delete_payment))
            .branch(case![Command::Cancel].endpoint(cancel_delete_payment))
            .branch(case![Command::ViewBalances].endpoint(block_delete_payment))
            .branch(case![Command::PayBack].endpoint(block_delete_payment))
            .branch(case![Command::ViewPayments].endpoint(block_delete_payment))
            .branch(case![Command::EditPayment { serial_num }].endpoint(block_delete_payment))
            .branch(
                case![Command::DeletePayment { serial_num }]
                    .endpoint(handle_repeated_delete_payment),
            ),
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
            case![State::EditPaymentDetails {
                payment,
                edited_payment,
                edit,
                payments,
                page
            }]
            .endpoint(action_edit_payment_edit),
        )
        .branch(case![State::AddConfirm { payment }].endpoint(callback_invalid_message))
        .branch(case![State::AddEditMenu { payment }].endpoint(callback_invalid_message))
        .branch(case![State::PayBackConfirm { payment }].endpoint(callback_invalid_message))
        .branch(
            case![State::EditPayment {
                payment,
                edited_payment,
                payments,
                page
            }]
            .endpoint(callback_invalid_message),
        )
        .branch(case![State::Start].endpoint(invalid_state));

    let callback_query_handler = Update::filter_callback_query()
        .branch(case![State::AddConfirm { payment }].endpoint(action_add_confirm))
        .branch(case![State::AddEditMenu { payment }].endpoint(action_add_edit_menu))
        .branch(case![State::PayBackConfirm { payment }].endpoint(action_pay_back_confirm))
        .branch(case![State::ViewPayments { payments, page }].endpoint(action_view_more))
        .branch(
            case![State::EditPayment {
                payment,
                edited_payment,
                payments,
                page
            }]
            .endpoint(action_edit_payment_confirm),
        )
        .branch(
            case![State::DeletePayment {
                payment,
                payments,
                page
            }]
            .endpoint(action_delete_payment_confirm),
        );

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
