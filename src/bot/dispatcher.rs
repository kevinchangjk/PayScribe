use teloxide::{
    dispatching::dialogue::{self, InMemStorage},
    prelude::*,
    types::MessageId,
    utils::command::BotCommands,
};

use crate::bot::handler::*;

use super::currency::Currency;

/* Dispatcher handles conversation branches with the user.
 * Bot states, commands, and control flow are defined here.
 */

#[derive(Clone, Default, Debug)]
pub enum State {
    #[default]
    Start,
    AddDescription {
        messages: Vec<MessageId>,
    },
    AddCreditor {
        messages: Vec<MessageId>,
        payment: AddPaymentParams,
    },
    AddTotal {
        messages: Vec<MessageId>,
        payment: AddPaymentParams,
    },
    AddDebtSelection {
        messages: Vec<MessageId>,
        payment: AddPaymentParams,
    },
    AddDebt {
        messages: Vec<MessageId>,
        payment: AddPaymentParams,
        debts_format: AddDebtsFormat,
    },
    AddConfirm {
        messages: Vec<MessageId>,
        payment: AddPaymentParams,
    },
    AddEditMenu {
        messages: Vec<MessageId>,
        payment: AddPaymentParams,
    },
    AddEditDebtsMenu {
        messages: Vec<MessageId>,
        payment: AddPaymentParams,
    },
    AddEdit {
        messages: Vec<MessageId>,
        payment: AddPaymentParams,
        edit: AddPaymentEdit,
    },
    PayBackCurrencyMenu {
        messages: Vec<MessageId>,
    },
    PayBackCurrency {
        messages: Vec<MessageId>,
    },
    PayBackDebts {
        messages: Vec<MessageId>,
        currency: Currency,
    },
    PayBackConfirm {
        messages: Vec<MessageId>,
        payment: PayBackParams,
    },
    ViewPayments {
        payments: Vec<Payment>,
        page: usize,
    },
    SelectPayment {
        messages: Vec<MessageId>,
        payments: Vec<Payment>,
        page: usize,
        function: SelectPaymentType,
    },
    EditPayment {
        messages: Vec<MessageId>,
        payment: Payment,
        edited_payment: EditPaymentParams,
        payments: Vec<Payment>,
        page: usize,
    },
    EditPaymentDebtSelection {
        messages: Vec<MessageId>,
        payment: Payment,
        edited_payment: EditPaymentParams,
        payments: Vec<Payment>,
        page: usize,
    },
    EditPaymentDetails {
        messages: Vec<MessageId>,
        payment: Payment,
        edited_payment: EditPaymentParams,
        edit: AddPaymentEdit,
        payments: Vec<Payment>,
        page: usize,
    },
    DeletePayment {
        messages: Vec<MessageId>,
        payment: Payment,
        payments: Vec<Payment>,
        page: usize,
    },
    BalancesMenu,
    SpendingsMenu,
    SettingsMenu {
        messages: Vec<MessageId>,
    },
    SettingsTimeZoneMenu {
        messages: Vec<MessageId>,
    },
    SettingsTimeZone {
        messages: Vec<MessageId>,
    },
    SettingsDefaultCurrencyMenu {
        messages: Vec<MessageId>,
    },
    SettingsDefaultCurrency {
        messages: Vec<MessageId>,
    },
    SettingsCurrencyConversion {
        messages: Vec<MessageId>,
    },
    SettingsEraseMessages {
        messages: Vec<MessageId>,
    },
}

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
pub enum Command {
    #[command(description = "Start me\\!")]
    Start,
    #[command(description = "Show this message")]
    Help,
    #[command(description = "Add a new payment")]
    AddPayment,
    #[command(description = "Add a record of paying back a debt")]
    PayBack,
    #[command(description = "View all payment records")]
    ViewPayments,
    #[command(description = "Edit a previous payment")]
    EditPayment,
    #[command(description = "Delete a previous payment")]
    DeletePayment,
    #[command(description = "View the current balances for everyone")]
    Balances,
    #[command(description = "View the total spendings for everyone")]
    Spendings,
    #[command(description = "View and edit my settings for everyone")]
    Settings,
    #[command(description = "Cancel whatever I'm doing")]
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
                .branch(case![Command::Cancel].endpoint(action_cancel))
                .branch(case![Command::AddPayment].endpoint(action_add_payment))
                .branch(case![Command::Balances].endpoint(action_view_balances))
                .branch(case![Command::PayBack].endpoint(action_pay_back))
                .branch(case![Command::ViewPayments].endpoint(action_view_payments))
                .branch(case![Command::EditPayment].endpoint(no_edit_payment))
                .branch(case![Command::DeletePayment].endpoint(no_delete_payment))
                .branch(case![Command::Settings].endpoint(action_settings))
                .branch(case![Command::Spendings].endpoint(action_view_spendings)),
        )
        .branch(
            case![State::AddDescription { messages }]
                .branch(case![Command::Start].endpoint(action_start))
                .branch(case![Command::Help].endpoint(action_help))
                .branch(case![Command::Cancel].endpoint(cancel_add_payment))
                .branch(case![Command::AddPayment].endpoint(handle_repeated_add_payment))
                .branch(case![Command::Balances].endpoint(block_add_payment))
                .branch(case![Command::PayBack].endpoint(block_add_payment))
                .branch(case![Command::ViewPayments].endpoint(block_add_payment))
                .branch(case![Command::EditPayment].endpoint(block_add_payment))
                .branch(case![Command::DeletePayment].endpoint(block_add_payment))
                .branch(case![Command::Settings].endpoint(block_add_payment))
                .branch(case![Command::Spendings].endpoint(block_add_payment)),
        )
        .branch(
            case![State::AddCreditor { messages, payment }]
                .branch(case![Command::Start].endpoint(action_start))
                .branch(case![Command::Help].endpoint(action_help))
                .branch(case![Command::Cancel].endpoint(cancel_add_payment))
                .branch(case![Command::AddPayment].endpoint(handle_repeated_add_payment))
                .branch(case![Command::Balances].endpoint(block_add_payment))
                .branch(case![Command::PayBack].endpoint(block_add_payment))
                .branch(case![Command::ViewPayments].endpoint(block_add_payment))
                .branch(case![Command::EditPayment].endpoint(block_add_payment))
                .branch(case![Command::DeletePayment].endpoint(block_add_payment))
                .branch(case![Command::Settings].endpoint(block_add_payment))
                .branch(case![Command::Spendings].endpoint(block_add_payment)),
        )
        .branch(
            case![State::AddTotal { messages, payment }]
                .branch(case![Command::Start].endpoint(action_start))
                .branch(case![Command::Help].endpoint(action_help))
                .branch(case![Command::Cancel].endpoint(cancel_add_payment))
                .branch(case![Command::AddPayment].endpoint(handle_repeated_add_payment))
                .branch(case![Command::Balances].endpoint(block_add_payment))
                .branch(case![Command::PayBack].endpoint(block_add_payment))
                .branch(case![Command::ViewPayments].endpoint(block_add_payment))
                .branch(case![Command::EditPayment].endpoint(block_add_payment))
                .branch(case![Command::DeletePayment].endpoint(block_add_payment))
                .branch(case![Command::Settings].endpoint(block_add_payment))
                .branch(case![Command::Spendings].endpoint(block_add_payment)),
        )
        .branch(
            case![State::AddDebtSelection { messages, payment }]
                .branch(case![Command::Start].endpoint(action_start))
                .branch(case![Command::Help].endpoint(action_help))
                .branch(case![Command::Cancel].endpoint(cancel_add_payment))
                .branch(case![Command::AddPayment].endpoint(handle_repeated_add_payment))
                .branch(case![Command::Balances].endpoint(block_add_payment))
                .branch(case![Command::PayBack].endpoint(block_add_payment))
                .branch(case![Command::ViewPayments].endpoint(block_add_payment))
                .branch(case![Command::EditPayment].endpoint(block_add_payment))
                .branch(case![Command::DeletePayment].endpoint(block_add_payment))
                .branch(case![Command::Settings].endpoint(block_add_payment))
                .branch(case![Command::Spendings].endpoint(block_add_payment)),
        )
        .branch(
            case![State::AddDebt {
                messages,
                payment,
                debts_format
            }]
            .branch(case![Command::Start].endpoint(action_start))
            .branch(case![Command::Help].endpoint(action_help))
            .branch(case![Command::Cancel].endpoint(cancel_add_payment))
            .branch(case![Command::AddPayment].endpoint(handle_repeated_add_payment))
            .branch(case![Command::Balances].endpoint(block_add_payment))
            .branch(case![Command::PayBack].endpoint(block_add_payment))
            .branch(case![Command::ViewPayments].endpoint(block_add_payment))
            .branch(case![Command::EditPayment].endpoint(block_add_payment))
            .branch(case![Command::DeletePayment].endpoint(block_add_payment))
            .branch(case![Command::Settings].endpoint(block_add_payment))
            .branch(case![Command::Spendings].endpoint(block_add_payment)),
        )
        .branch(
            case![State::AddConfirm { messages, payment }]
                .branch(case![Command::Start].endpoint(action_start))
                .branch(case![Command::Help].endpoint(action_help))
                .branch(case![Command::Cancel].endpoint(cancel_add_payment))
                .branch(case![Command::AddPayment].endpoint(handle_repeated_add_payment))
                .branch(case![Command::Balances].endpoint(block_add_payment))
                .branch(case![Command::PayBack].endpoint(block_add_payment))
                .branch(case![Command::ViewPayments].endpoint(block_add_payment))
                .branch(case![Command::EditPayment].endpoint(block_add_payment))
                .branch(case![Command::DeletePayment].endpoint(block_add_payment))
                .branch(case![Command::Settings].endpoint(block_add_payment))
                .branch(case![Command::Spendings].endpoint(block_add_payment)),
        )
        .branch(
            case![State::AddEditMenu { messages, payment }]
                .branch(case![Command::Start].endpoint(action_start))
                .branch(case![Command::Help].endpoint(action_help))
                .branch(case![Command::Cancel].endpoint(cancel_add_payment))
                .branch(case![Command::AddPayment].endpoint(handle_repeated_add_payment))
                .branch(case![Command::Balances].endpoint(block_add_payment))
                .branch(case![Command::PayBack].endpoint(block_add_payment))
                .branch(case![Command::ViewPayments].endpoint(block_add_payment))
                .branch(case![Command::EditPayment].endpoint(block_add_payment))
                .branch(case![Command::DeletePayment].endpoint(block_add_payment))
                .branch(case![Command::Settings].endpoint(block_add_payment))
                .branch(case![Command::Spendings].endpoint(block_add_payment)),
        )
        .branch(
            case![State::AddEdit {
                messages,
                payment,
                edit
            }]
            .branch(case![Command::Start].endpoint(action_start))
            .branch(case![Command::Help].endpoint(action_help))
            .branch(case![Command::Cancel].endpoint(cancel_add_payment))
            .branch(case![Command::AddPayment].endpoint(handle_repeated_add_payment))
            .branch(case![Command::Balances].endpoint(block_add_payment))
            .branch(case![Command::PayBack].endpoint(block_add_payment))
            .branch(case![Command::ViewPayments].endpoint(block_add_payment))
            .branch(case![Command::EditPayment].endpoint(block_add_payment))
            .branch(case![Command::DeletePayment].endpoint(block_add_payment))
            .branch(case![Command::Settings].endpoint(block_add_payment))
            .branch(case![Command::Spendings].endpoint(block_add_payment)),
        )
        .branch(
            case![State::PayBackCurrencyMenu { messages }]
                .branch(case![Command::Start].endpoint(action_start))
                .branch(case![Command::Help].endpoint(action_help))
                .branch(case![Command::Cancel].endpoint(cancel_pay_back))
                .branch(case![Command::AddPayment].endpoint(block_pay_back))
                .branch(case![Command::Balances].endpoint(block_pay_back))
                .branch(case![Command::PayBack].endpoint(handle_repeated_pay_back))
                .branch(case![Command::ViewPayments].endpoint(block_pay_back))
                .branch(case![Command::EditPayment].endpoint(block_pay_back))
                .branch(case![Command::DeletePayment].endpoint(block_pay_back))
                .branch(case![Command::Settings].endpoint(block_pay_back))
                .branch(case![Command::Spendings].endpoint(block_pay_back)),
        )
        .branch(
            case![State::PayBackCurrency { messages }]
                .branch(case![Command::Start].endpoint(action_start))
                .branch(case![Command::Help].endpoint(action_help))
                .branch(case![Command::Cancel].endpoint(cancel_pay_back))
                .branch(case![Command::AddPayment].endpoint(block_pay_back))
                .branch(case![Command::Balances].endpoint(block_pay_back))
                .branch(case![Command::PayBack].endpoint(handle_repeated_pay_back))
                .branch(case![Command::ViewPayments].endpoint(block_pay_back))
                .branch(case![Command::EditPayment].endpoint(block_pay_back))
                .branch(case![Command::DeletePayment].endpoint(block_pay_back))
                .branch(case![Command::Settings].endpoint(block_pay_back))
                .branch(case![Command::Spendings].endpoint(block_pay_back)),
        )
        .branch(
            case![State::PayBackDebts { messages, currency }]
                .branch(case![Command::Start].endpoint(action_start))
                .branch(case![Command::Help].endpoint(action_help))
                .branch(case![Command::Cancel].endpoint(cancel_pay_back))
                .branch(case![Command::AddPayment].endpoint(block_pay_back))
                .branch(case![Command::Balances].endpoint(block_pay_back))
                .branch(case![Command::PayBack].endpoint(handle_repeated_pay_back))
                .branch(case![Command::ViewPayments].endpoint(block_pay_back))
                .branch(case![Command::EditPayment].endpoint(block_pay_back))
                .branch(case![Command::DeletePayment].endpoint(block_pay_back))
                .branch(case![Command::Settings].endpoint(block_pay_back))
                .branch(case![Command::Spendings].endpoint(block_pay_back)),
        )
        .branch(
            case![State::PayBackConfirm { messages, payment }]
                .branch(case![Command::Start].endpoint(action_start))
                .branch(case![Command::Help].endpoint(action_help))
                .branch(case![Command::Cancel].endpoint(cancel_pay_back))
                .branch(case![Command::AddPayment].endpoint(block_pay_back))
                .branch(case![Command::Balances].endpoint(block_pay_back))
                .branch(case![Command::PayBack].endpoint(handle_repeated_pay_back))
                .branch(case![Command::ViewPayments].endpoint(block_pay_back))
                .branch(case![Command::EditPayment].endpoint(block_pay_back))
                .branch(case![Command::DeletePayment].endpoint(block_pay_back))
                .branch(case![Command::Settings].endpoint(block_pay_back))
                .branch(case![Command::Spendings].endpoint(block_pay_back)),
        )
        .branch(
            case![State::ViewPayments { payments, page }]
                .branch(case![Command::Start].endpoint(action_start))
                .branch(case![Command::Help].endpoint(action_help))
                .branch(case![Command::Cancel].endpoint(action_cancel))
                .branch(case![Command::AddPayment].endpoint(action_add_payment))
                .branch(case![Command::Balances].endpoint(action_view_balances))
                .branch(case![Command::PayBack].endpoint(action_pay_back))
                .branch(case![Command::ViewPayments].endpoint(action_view_payments))
                .branch(case![Command::EditPayment].endpoint(action_select_payment_edit))
                .branch(case![Command::DeletePayment].endpoint(action_select_payment_delete))
                .branch(case![Command::Settings].endpoint(action_settings))
                .branch(case![Command::Spendings].endpoint(action_view_spendings)),
        )
        .branch(
            case![State::SelectPayment {
                messages,
                payments,
                page,
                function
            }]
            .branch(case![Command::Start].endpoint(action_start))
            .branch(case![Command::Help].endpoint(action_help))
            .branch(case![Command::Cancel].endpoint(cancel_select_payment))
            .branch(case![Command::AddPayment].endpoint(block_select_payment))
            .branch(case![Command::Balances].endpoint(block_select_payment))
            .branch(case![Command::PayBack].endpoint(block_select_payment))
            .branch(case![Command::ViewPayments].endpoint(block_select_payment))
            .branch(case![Command::EditPayment].endpoint(handle_repeated_select_payment))
            .branch(case![Command::DeletePayment].endpoint(handle_repeated_select_payment))
            .branch(case![Command::Settings].endpoint(block_select_payment))
            .branch(case![Command::Spendings].endpoint(block_select_payment)),
        )
        .branch(
            case![State::EditPayment {
                messages,
                payment,
                edited_payment,
                payments,
                page
            }]
            .branch(case![Command::Start].endpoint(action_start))
            .branch(case![Command::Help].endpoint(action_help))
            .branch(case![Command::Cancel].endpoint(cancel_edit_payment))
            .branch(case![Command::AddPayment].endpoint(block_edit_payment))
            .branch(case![Command::Balances].endpoint(block_edit_payment))
            .branch(case![Command::PayBack].endpoint(block_edit_payment))
            .branch(case![Command::ViewPayments].endpoint(block_edit_payment))
            .branch(case![Command::EditPayment].endpoint(handle_repeated_edit_payment))
            .branch(case![Command::DeletePayment].endpoint(block_edit_payment))
            .branch(case![Command::Settings].endpoint(block_edit_payment))
            .branch(case![Command::Spendings].endpoint(block_edit_payment)),
        )
        .branch(
            case![State::EditPaymentDebtSelection {
                messages,
                payment,
                edited_payment,
                payments,
                page
            }]
            .branch(case![Command::Start].endpoint(action_start))
            .branch(case![Command::Help].endpoint(action_help))
            .branch(case![Command::Cancel].endpoint(cancel_edit_payment))
            .branch(case![Command::AddPayment].endpoint(block_edit_payment))
            .branch(case![Command::Balances].endpoint(block_edit_payment))
            .branch(case![Command::PayBack].endpoint(block_edit_payment))
            .branch(case![Command::ViewPayments].endpoint(block_edit_payment))
            .branch(case![Command::EditPayment].endpoint(handle_repeated_edit_payment))
            .branch(case![Command::DeletePayment].endpoint(block_edit_payment))
            .branch(case![Command::Settings].endpoint(block_edit_payment))
            .branch(case![Command::Spendings].endpoint(block_edit_payment)),
        )
        .branch(
            case![State::EditPaymentDetails {
                messages,
                payment,
                edited_payment,
                edit,
                payments,
                page
            }]
            .branch(case![Command::Start].endpoint(action_start))
            .branch(case![Command::Help].endpoint(action_help))
            .branch(case![Command::Cancel].endpoint(cancel_edit_payment))
            .branch(case![Command::AddPayment].endpoint(block_edit_payment))
            .branch(case![Command::Balances].endpoint(block_edit_payment))
            .branch(case![Command::PayBack].endpoint(block_edit_payment))
            .branch(case![Command::ViewPayments].endpoint(block_edit_payment))
            .branch(case![Command::EditPayment].endpoint(handle_repeated_edit_payment))
            .branch(case![Command::DeletePayment].endpoint(block_edit_payment))
            .branch(case![Command::Settings].endpoint(block_edit_payment))
            .branch(case![Command::Spendings].endpoint(block_edit_payment)),
        )
        .branch(
            case![State::DeletePayment {
                messages,
                payment,
                payments,
                page
            }]
            .branch(case![Command::Start].endpoint(action_start))
            .branch(case![Command::Help].endpoint(action_help))
            .branch(case![Command::Cancel].endpoint(cancel_delete_payment))
            .branch(case![Command::AddPayment].endpoint(block_delete_payment))
            .branch(case![Command::Balances].endpoint(block_delete_payment))
            .branch(case![Command::PayBack].endpoint(block_delete_payment))
            .branch(case![Command::ViewPayments].endpoint(block_delete_payment))
            .branch(case![Command::EditPayment].endpoint(block_delete_payment))
            .branch(case![Command::DeletePayment].endpoint(handle_repeated_delete_payment))
            .branch(case![Command::Settings].endpoint(block_delete_payment))
            .branch(case![Command::Spendings].endpoint(block_delete_payment)),
        )
        .branch(
            case![State::SettingsMenu { messages }]
                .branch(case![Command::Start].endpoint(action_start))
                .branch(case![Command::Help].endpoint(action_help))
                .branch(case![Command::Cancel].endpoint(cancel_settings))
                .branch(case![Command::AddPayment].endpoint(block_settings))
                .branch(case![Command::Balances].endpoint(block_settings))
                .branch(case![Command::PayBack].endpoint(block_settings))
                .branch(case![Command::ViewPayments].endpoint(block_settings))
                .branch(case![Command::EditPayment].endpoint(block_settings))
                .branch(case![Command::DeletePayment].endpoint(block_settings))
                .branch(case![Command::Settings].endpoint(handle_repeated_settings))
                .branch(case![Command::Spendings].endpoint(block_settings)),
        )
        .branch(
            case![State::SettingsTimeZoneMenu { messages }]
                .branch(case![Command::Start].endpoint(action_start))
                .branch(case![Command::Help].endpoint(action_help))
                .branch(case![Command::Cancel].endpoint(cancel_settings))
                .branch(case![Command::AddPayment].endpoint(block_settings))
                .branch(case![Command::Balances].endpoint(block_settings))
                .branch(case![Command::PayBack].endpoint(block_settings))
                .branch(case![Command::ViewPayments].endpoint(block_settings))
                .branch(case![Command::EditPayment].endpoint(block_settings))
                .branch(case![Command::DeletePayment].endpoint(block_settings))
                .branch(case![Command::Settings].endpoint(handle_repeated_settings))
                .branch(case![Command::Spendings].endpoint(block_settings)),
        )
        .branch(
            case![State::SettingsTimeZone { messages }]
                .branch(case![Command::Start].endpoint(action_start))
                .branch(case![Command::Help].endpoint(action_help))
                .branch(case![Command::Cancel].endpoint(cancel_settings))
                .branch(case![Command::AddPayment].endpoint(block_settings))
                .branch(case![Command::Balances].endpoint(block_settings))
                .branch(case![Command::PayBack].endpoint(block_settings))
                .branch(case![Command::ViewPayments].endpoint(block_settings))
                .branch(case![Command::EditPayment].endpoint(block_settings))
                .branch(case![Command::DeletePayment].endpoint(block_settings))
                .branch(case![Command::Settings].endpoint(handle_repeated_settings))
                .branch(case![Command::Spendings].endpoint(block_settings)),
        )
        .branch(
            case![State::SettingsDefaultCurrencyMenu { messages }]
                .branch(case![Command::Start].endpoint(action_start))
                .branch(case![Command::Help].endpoint(action_help))
                .branch(case![Command::Cancel].endpoint(cancel_settings))
                .branch(case![Command::AddPayment].endpoint(block_settings))
                .branch(case![Command::Balances].endpoint(block_settings))
                .branch(case![Command::PayBack].endpoint(block_settings))
                .branch(case![Command::ViewPayments].endpoint(block_settings))
                .branch(case![Command::EditPayment].endpoint(block_settings))
                .branch(case![Command::DeletePayment].endpoint(block_settings))
                .branch(case![Command::Settings].endpoint(handle_repeated_settings))
                .branch(case![Command::Spendings].endpoint(block_settings)),
        )
        .branch(
            case![State::SettingsDefaultCurrency { messages }]
                .branch(case![Command::Start].endpoint(action_start))
                .branch(case![Command::Help].endpoint(action_help))
                .branch(case![Command::Cancel].endpoint(cancel_settings))
                .branch(case![Command::AddPayment].endpoint(block_settings))
                .branch(case![Command::Balances].endpoint(block_settings))
                .branch(case![Command::PayBack].endpoint(block_settings))
                .branch(case![Command::ViewPayments].endpoint(block_settings))
                .branch(case![Command::EditPayment].endpoint(block_settings))
                .branch(case![Command::DeletePayment].endpoint(block_settings))
                .branch(case![Command::Settings].endpoint(handle_repeated_settings))
                .branch(case![Command::Spendings].endpoint(block_settings)),
        )
        .branch(
            case![State::SettingsCurrencyConversion { messages }]
                .branch(case![Command::Start].endpoint(action_start))
                .branch(case![Command::Help].endpoint(action_help))
                .branch(case![Command::Cancel].endpoint(cancel_settings))
                .branch(case![Command::AddPayment].endpoint(block_settings))
                .branch(case![Command::Balances].endpoint(block_settings))
                .branch(case![Command::PayBack].endpoint(block_settings))
                .branch(case![Command::ViewPayments].endpoint(block_settings))
                .branch(case![Command::EditPayment].endpoint(block_settings))
                .branch(case![Command::DeletePayment].endpoint(block_settings))
                .branch(case![Command::Settings].endpoint(handle_repeated_settings))
                .branch(case![Command::Spendings].endpoint(block_settings)),
        )
        .branch(
            case![State::SettingsEraseMessages { messages }]
                .branch(case![Command::Start].endpoint(action_start))
                .branch(case![Command::Help].endpoint(action_help))
                .branch(case![Command::Cancel].endpoint(cancel_settings))
                .branch(case![Command::AddPayment].endpoint(block_settings))
                .branch(case![Command::Balances].endpoint(block_settings))
                .branch(case![Command::PayBack].endpoint(block_settings))
                .branch(case![Command::ViewPayments].endpoint(block_settings))
                .branch(case![Command::EditPayment].endpoint(block_settings))
                .branch(case![Command::DeletePayment].endpoint(block_settings))
                .branch(case![Command::Settings].endpoint(handle_repeated_settings))
                .branch(case![Command::Spendings].endpoint(block_settings)),
        )
        .branch(
            case![State::BalancesMenu]
                .branch(case![Command::Start].endpoint(action_start))
                .branch(case![Command::Help].endpoint(action_help))
                .branch(case![Command::Cancel].endpoint(action_cancel))
                .branch(case![Command::AddPayment].endpoint(action_add_payment))
                .branch(case![Command::Balances].endpoint(action_view_balances))
                .branch(case![Command::PayBack].endpoint(action_pay_back))
                .branch(case![Command::ViewPayments].endpoint(action_view_payments))
                .branch(case![Command::EditPayment].endpoint(no_edit_payment))
                .branch(case![Command::DeletePayment].endpoint(no_delete_payment))
                .branch(case![Command::Settings].endpoint(action_settings))
                .branch(case![Command::Spendings].endpoint(action_view_spendings)),
        )
        .branch(
            case![State::SpendingsMenu]
                .branch(case![Command::Start].endpoint(action_start))
                .branch(case![Command::Help].endpoint(action_help))
                .branch(case![Command::Cancel].endpoint(action_cancel))
                .branch(case![Command::AddPayment].endpoint(action_add_payment))
                .branch(case![Command::Balances].endpoint(action_view_balances))
                .branch(case![Command::PayBack].endpoint(action_pay_back))
                .branch(case![Command::ViewPayments].endpoint(action_view_payments))
                .branch(case![Command::EditPayment].endpoint(no_edit_payment))
                .branch(case![Command::DeletePayment].endpoint(no_delete_payment))
                .branch(case![Command::Settings].endpoint(action_settings))
                .branch(case![Command::Spendings].endpoint(action_view_spendings)),
        );

    let message_handler = Update::filter_message()
        .branch(command_handler)
        .branch(case![State::AddDescription { messages }].endpoint(action_add_description))
        .branch(case![State::AddCreditor { messages, payment }].endpoint(action_add_creditor))
        .branch(case![State::AddTotal { messages, payment }].endpoint(action_add_total))
        .branch(
            case![State::AddDebt {
                messages,
                payment,
                debts_format
            }]
            .endpoint(action_add_debt),
        )
        .branch(
            case![State::AddEdit {
                messages,
                payment,
                edit
            }]
            .endpoint(action_add_edit),
        )
        .branch(case![State::PayBackCurrency { messages }].endpoint(action_pay_back_currency))
        .branch(case![State::PayBackDebts { messages, currency }].endpoint(action_pay_back_debts))
        .branch(
            case![State::EditPaymentDetails {
                messages,
                payment,
                edited_payment,
                edit,
                payments,
                page
            }]
            .endpoint(action_edit_payment_edit),
        )
        .branch(case![State::SettingsTimeZone { messages }].endpoint(action_settings_time_zone))
        .branch(
            case![State::SettingsDefaultCurrency { messages }]
                .endpoint(action_settings_default_currency),
        )
        .branch(
            case![State::AddDebtSelection { messages, payment }].endpoint(callback_invalid_message),
        )
        .branch(case![State::AddConfirm { messages, payment }].endpoint(callback_invalid_message))
        .branch(
            case![State::AddEditDebtsMenu { messages, payment }].endpoint(callback_invalid_message),
        )
        .branch(case![State::AddEditMenu { messages, payment }].endpoint(callback_invalid_message))
        .branch(case![State::PayBackCurrencyMenu { messages }].endpoint(callback_invalid_message))
        .branch(
            case![State::PayBackConfirm { messages, payment }].endpoint(callback_invalid_message),
        )
        .branch(
            case![State::SelectPayment {
                messages,
                payments,
                page,
                function
            }]
            .endpoint(callback_invalid_message),
        )
        .branch(
            case![State::EditPayment {
                messages,
                payment,
                edited_payment,
                payments,
                page
            }]
            .endpoint(callback_invalid_message),
        )
        .branch(
            case![State::EditPaymentDebtSelection {
                messages,
                payment,
                edited_payment,
                payments,
                page
            }]
            .endpoint(callback_invalid_message),
        )
        .branch(
            case![State::DeletePayment {
                messages,
                payment,
                payments,
                page
            }]
            .endpoint(callback_invalid_message),
        )
        .branch(case![State::SettingsMenu { messages }].endpoint(callback_invalid_message))
        .branch(case![State::SettingsTimeZoneMenu { messages }].endpoint(callback_invalid_message))
        .branch(
            case![State::SettingsDefaultCurrencyMenu { messages }]
                .endpoint(callback_invalid_message),
        )
        .branch(
            case![State::SettingsCurrencyConversion { messages }]
                .endpoint(callback_invalid_message),
        )
        .branch(case![State::SettingsEraseMessages { messages }].endpoint(callback_invalid_message))
        .branch(case![State::ViewPayments { payments, page }].endpoint(invalid_state))
        .branch(case![State::BalancesMenu].endpoint(invalid_state))
        .branch(case![State::SpendingsMenu].endpoint(invalid_state))
        .branch(case![State::Start].endpoint(invalid_state));

    let callback_query_handler = Update::filter_callback_query()
        .branch(
            case![State::AddDebtSelection { messages, payment }]
                .endpoint(action_add_debt_selection),
        )
        .branch(case![State::AddConfirm { messages, payment }].endpoint(action_add_confirm))
        .branch(
            case![State::AddEditDebtsMenu { messages, payment }]
                .endpoint(action_add_debt_selection),
        )
        .branch(case![State::AddEditMenu { messages, payment }].endpoint(action_add_edit_menu))
        .branch(
            case![State::PayBackCurrencyMenu { messages }].endpoint(action_pay_back_currency_menu),
        )
        .branch(
            case![State::PayBackConfirm { messages, payment }].endpoint(action_pay_back_confirm),
        )
        .branch(case![State::ViewPayments { payments, page }].endpoint(action_view_more))
        .branch(
            case![State::SelectPayment {
                messages,
                payments,
                page,
                function
            }]
            .endpoint(action_select_payment_number),
        )
        .branch(
            case![State::EditPayment {
                messages,
                payment,
                edited_payment,
                payments,
                page
            }]
            .endpoint(action_edit_payment_confirm),
        )
        .branch(
            case![State::EditPaymentDebtSelection {
                messages,
                payment,
                edited_payment,
                payments,
                page
            }]
            .endpoint(action_edit_payment_debts),
        )
        .branch(
            case![State::DeletePayment {
                messages,
                payment,
                payments,
                page
            }]
            .endpoint(action_delete_payment_confirm),
        )
        .branch(case![State::BalancesMenu].endpoint(action_balances_menu))
        .branch(case![State::SpendingsMenu].endpoint(action_spendings_menu))
        .branch(case![State::SettingsMenu { messages }].endpoint(action_settings_menu))
        .branch(case![State::SettingsTimeZoneMenu { messages }].endpoint(action_time_zone_menu))
        .branch(
            case![State::SettingsDefaultCurrencyMenu { messages }]
                .endpoint(action_default_currency_menu),
        )
        .branch(
            case![State::SettingsCurrencyConversion { messages }]
                .endpoint(action_settings_currency_conversion),
        )
        .branch(
            case![State::SettingsEraseMessages { messages }]
                .endpoint(action_settings_erase_messages),
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
