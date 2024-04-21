use teloxide::{
    dispatching::{dialogue, dialogue::InMemStorage},
    prelude::*,
    utils::command::BotCommands,
};

use crate::bot::handler::*;

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
    AddDebtSelection {
        payment: AddPaymentParams,
    },
    AddDebt {
        payment: AddPaymentParams,
        debts_format: AddDebtsFormat,
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
    AddEditDebtsMenu {
        payment: AddPaymentParams,
    },
    AddEdit {
        payment: AddPaymentParams,
        edit: AddPaymentEdit,
    },
    PayBackCurrencyMenu,
    PayBackCurrency,
    PayBackDebts {
        currency: Currency,
    },
    PayBackConfirm {
        payment: PayBackParams,
    },
    ViewPayments {
        payments: Vec<Payment>,
        page: usize,
    },
    SelectPayment {
        payments: Vec<Payment>,
        page: usize,
        function: SelectPaymentType,
    },
    EditPayment {
        payment: Payment,
        edited_payment: EditPaymentParams,
        payments: Vec<Payment>,
        page: usize,
    },
    EditPaymentDebtSelection {
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
    SettingsMenu,
    SettingsTimeZoneMenu,
    SettingsTimeZone,
    SettingsDefaultCurrencyMenu,
    SettingsDefaultCurrency,
    SettingsCurrencyConversion,
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
    #[command(description = "Add an entry paying back others in the group")]
    PayBack,
    #[command(description = "View all payment records for the group")]
    ViewPayments,
    #[command(description = "Edit a payment record previously added")]
    EditPayment,
    #[command(description = "Delete a payment record previously added")]
    DeletePayment,
    #[command(description = "View the current balances for the group")]
    Balances,
    #[command(description = "View and edit bot settings for the chat")]
    Settings,
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
                .branch(case![Command::Cancel].endpoint(action_cancel))
                .branch(case![Command::AddPayment].endpoint(action_add_payment))
                .branch(case![Command::Balances].endpoint(action_view_balances))
                .branch(case![Command::PayBack].endpoint(action_pay_back))
                .branch(case![Command::ViewPayments].endpoint(action_view_payments))
                .branch(case![Command::EditPayment].endpoint(no_edit_payment))
                .branch(case![Command::DeletePayment].endpoint(no_delete_payment))
                .branch(case![Command::Settings].endpoint(action_settings)),
        )
        .branch(
            case![State::AddDescription]
                .branch(case![Command::Start].endpoint(action_start))
                .branch(case![Command::Help].endpoint(action_help))
                .branch(case![Command::Cancel].endpoint(cancel_add_payment))
                .branch(case![Command::AddPayment].endpoint(handle_repeated_add_payment))
                .branch(case![Command::Balances].endpoint(block_add_payment))
                .branch(case![Command::PayBack].endpoint(block_add_payment))
                .branch(case![Command::ViewPayments].endpoint(block_add_payment))
                .branch(case![Command::EditPayment].endpoint(block_add_payment))
                .branch(case![Command::DeletePayment].endpoint(block_add_payment))
                .branch(case![Command::Settings].endpoint(block_add_payment)),
        )
        .branch(
            case![State::AddCreditor { payment }]
                .branch(case![Command::Start].endpoint(action_start))
                .branch(case![Command::Help].endpoint(action_help))
                .branch(case![Command::Cancel].endpoint(cancel_add_payment))
                .branch(case![Command::AddPayment].endpoint(handle_repeated_add_payment))
                .branch(case![Command::Balances].endpoint(block_add_payment))
                .branch(case![Command::PayBack].endpoint(block_add_payment))
                .branch(case![Command::ViewPayments].endpoint(block_add_payment))
                .branch(case![Command::EditPayment].endpoint(block_add_payment))
                .branch(case![Command::DeletePayment].endpoint(block_add_payment))
                .branch(case![Command::Settings].endpoint(block_add_payment)),
        )
        .branch(
            case![State::AddTotal { payment }]
                .branch(case![Command::Start].endpoint(action_start))
                .branch(case![Command::Help].endpoint(action_help))
                .branch(case![Command::Cancel].endpoint(cancel_add_payment))
                .branch(case![Command::AddPayment].endpoint(handle_repeated_add_payment))
                .branch(case![Command::Balances].endpoint(block_add_payment))
                .branch(case![Command::PayBack].endpoint(block_add_payment))
                .branch(case![Command::ViewPayments].endpoint(block_add_payment))
                .branch(case![Command::EditPayment].endpoint(block_add_payment))
                .branch(case![Command::DeletePayment].endpoint(block_add_payment))
                .branch(case![Command::Settings].endpoint(block_add_payment)),
        )
        .branch(
            case![State::AddDebtSelection { payment }]
                .branch(case![Command::Start].endpoint(action_start))
                .branch(case![Command::Help].endpoint(action_help))
                .branch(case![Command::Cancel].endpoint(cancel_add_payment))
                .branch(case![Command::AddPayment].endpoint(handle_repeated_add_payment))
                .branch(case![Command::Balances].endpoint(block_add_payment))
                .branch(case![Command::PayBack].endpoint(block_add_payment))
                .branch(case![Command::ViewPayments].endpoint(block_add_payment))
                .branch(case![Command::EditPayment].endpoint(block_add_payment))
                .branch(case![Command::DeletePayment].endpoint(block_add_payment))
                .branch(case![Command::Settings].endpoint(block_add_payment)),
        )
        .branch(
            case![State::AddDebt {
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
            .branch(case![Command::Settings].endpoint(block_add_payment)),
        )
        .branch(
            case![State::AddConfirm { payment }]
                .branch(case![Command::Start].endpoint(action_start))
                .branch(case![Command::Help].endpoint(action_help))
                .branch(case![Command::Cancel].endpoint(cancel_add_payment))
                .branch(case![Command::AddPayment].endpoint(handle_repeated_add_payment))
                .branch(case![Command::Balances].endpoint(block_add_payment))
                .branch(case![Command::PayBack].endpoint(block_add_payment))
                .branch(case![Command::ViewPayments].endpoint(block_add_payment))
                .branch(case![Command::EditPayment].endpoint(block_add_payment))
                .branch(case![Command::DeletePayment].endpoint(block_add_payment))
                .branch(case![Command::Settings].endpoint(block_add_payment)),
        )
        .branch(
            case![State::AddEditMenu { payment }]
                .branch(case![Command::Start].endpoint(action_start))
                .branch(case![Command::Help].endpoint(action_help))
                .branch(case![Command::Cancel].endpoint(cancel_add_payment))
                .branch(case![Command::AddPayment].endpoint(handle_repeated_add_payment))
                .branch(case![Command::Balances].endpoint(block_add_payment))
                .branch(case![Command::PayBack].endpoint(block_add_payment))
                .branch(case![Command::ViewPayments].endpoint(block_add_payment))
                .branch(case![Command::EditPayment].endpoint(block_add_payment))
                .branch(case![Command::DeletePayment].endpoint(block_add_payment))
                .branch(case![Command::Settings].endpoint(block_add_payment)),
        )
        .branch(
            case![State::AddEdit { payment, edit }]
                .branch(case![Command::Start].endpoint(action_start))
                .branch(case![Command::Help].endpoint(action_help))
                .branch(case![Command::Cancel].endpoint(cancel_add_payment))
                .branch(case![Command::AddPayment].endpoint(handle_repeated_add_payment))
                .branch(case![Command::Balances].endpoint(block_add_payment))
                .branch(case![Command::PayBack].endpoint(block_add_payment))
                .branch(case![Command::ViewPayments].endpoint(block_add_payment))
                .branch(case![Command::EditPayment].endpoint(block_add_payment))
                .branch(case![Command::DeletePayment].endpoint(block_add_payment))
                .branch(case![Command::Settings].endpoint(block_add_payment)),
        )
        .branch(
            case![State::PayBackCurrencyMenu]
                .branch(case![Command::Start].endpoint(action_start))
                .branch(case![Command::Help].endpoint(action_help))
                .branch(case![Command::Cancel].endpoint(cancel_pay_back))
                .branch(case![Command::AddPayment].endpoint(block_pay_back))
                .branch(case![Command::Balances].endpoint(block_pay_back))
                .branch(case![Command::PayBack].endpoint(handle_repeated_pay_back))
                .branch(case![Command::ViewPayments].endpoint(block_pay_back))
                .branch(case![Command::EditPayment].endpoint(block_pay_back))
                .branch(case![Command::DeletePayment].endpoint(block_pay_back))
                .branch(case![Command::Settings].endpoint(block_pay_back)),
        )
        .branch(
            case![State::PayBackCurrency]
                .branch(case![Command::Start].endpoint(action_start))
                .branch(case![Command::Help].endpoint(action_help))
                .branch(case![Command::Cancel].endpoint(cancel_pay_back))
                .branch(case![Command::AddPayment].endpoint(block_pay_back))
                .branch(case![Command::Balances].endpoint(block_pay_back))
                .branch(case![Command::PayBack].endpoint(handle_repeated_pay_back))
                .branch(case![Command::ViewPayments].endpoint(block_pay_back))
                .branch(case![Command::EditPayment].endpoint(block_pay_back))
                .branch(case![Command::DeletePayment].endpoint(block_pay_back))
                .branch(case![Command::Settings].endpoint(block_pay_back)),
        )
        .branch(
            case![State::PayBackDebts { currency }]
                .branch(case![Command::Start].endpoint(action_start))
                .branch(case![Command::Help].endpoint(action_help))
                .branch(case![Command::Cancel].endpoint(cancel_pay_back))
                .branch(case![Command::AddPayment].endpoint(block_pay_back))
                .branch(case![Command::Balances].endpoint(block_pay_back))
                .branch(case![Command::PayBack].endpoint(handle_repeated_pay_back))
                .branch(case![Command::ViewPayments].endpoint(block_pay_back))
                .branch(case![Command::EditPayment].endpoint(block_pay_back))
                .branch(case![Command::DeletePayment].endpoint(block_pay_back))
                .branch(case![Command::Settings].endpoint(block_pay_back)),
        )
        .branch(
            case![State::PayBackConfirm { payment }]
                .branch(case![Command::Start].endpoint(action_start))
                .branch(case![Command::Help].endpoint(action_help))
                .branch(case![Command::Cancel].endpoint(cancel_pay_back))
                .branch(case![Command::AddPayment].endpoint(block_pay_back))
                .branch(case![Command::Balances].endpoint(block_pay_back))
                .branch(case![Command::PayBack].endpoint(handle_repeated_pay_back))
                .branch(case![Command::ViewPayments].endpoint(block_pay_back))
                .branch(case![Command::EditPayment].endpoint(block_pay_back))
                .branch(case![Command::DeletePayment].endpoint(block_pay_back))
                .branch(case![Command::Settings].endpoint(block_pay_back)),
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
                .branch(case![Command::Settings].endpoint(action_settings)),
        )
        .branch(
            case![State::SelectPayment {
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
            .branch(case![Command::Settings].endpoint(block_select_payment)),
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
            .branch(case![Command::Cancel].endpoint(cancel_edit_payment))
            .branch(case![Command::AddPayment].endpoint(block_edit_payment))
            .branch(case![Command::Balances].endpoint(block_edit_payment))
            .branch(case![Command::PayBack].endpoint(block_edit_payment))
            .branch(case![Command::ViewPayments].endpoint(block_edit_payment))
            .branch(case![Command::EditPayment].endpoint(handle_repeated_edit_payment))
            .branch(case![Command::DeletePayment].endpoint(block_edit_payment))
            .branch(case![Command::Settings].endpoint(block_edit_payment)),
        )
        .branch(
            case![State::EditPaymentDebtSelection {
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
            .branch(case![Command::Settings].endpoint(block_edit_payment)),
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
            .branch(case![Command::Cancel].endpoint(cancel_edit_payment))
            .branch(case![Command::AddPayment].endpoint(block_edit_payment))
            .branch(case![Command::Balances].endpoint(block_edit_payment))
            .branch(case![Command::PayBack].endpoint(block_edit_payment))
            .branch(case![Command::ViewPayments].endpoint(block_edit_payment))
            .branch(case![Command::EditPayment].endpoint(handle_repeated_edit_payment))
            .branch(case![Command::DeletePayment].endpoint(block_edit_payment))
            .branch(case![Command::Settings].endpoint(block_edit_payment)),
        )
        .branch(
            case![State::DeletePayment {
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
            .branch(case![Command::Settings].endpoint(block_delete_payment)),
        )
        .branch(
            case![State::SettingsMenu]
                .branch(case![Command::Start].endpoint(action_start))
                .branch(case![Command::Help].endpoint(action_help))
                .branch(case![Command::Cancel].endpoint(cancel_settings))
                .branch(case![Command::AddPayment].endpoint(block_settings))
                .branch(case![Command::Balances].endpoint(block_settings))
                .branch(case![Command::PayBack].endpoint(block_settings))
                .branch(case![Command::ViewPayments].endpoint(block_settings))
                .branch(case![Command::EditPayment].endpoint(block_settings))
                .branch(case![Command::DeletePayment].endpoint(block_settings))
                .branch(case![Command::Settings].endpoint(handle_repeated_settings)),
        )
        .branch(
            case![State::SettingsTimeZoneMenu]
                .branch(case![Command::Start].endpoint(action_start))
                .branch(case![Command::Help].endpoint(action_help))
                .branch(case![Command::Cancel].endpoint(cancel_settings))
                .branch(case![Command::AddPayment].endpoint(block_settings))
                .branch(case![Command::Balances].endpoint(block_settings))
                .branch(case![Command::PayBack].endpoint(block_settings))
                .branch(case![Command::ViewPayments].endpoint(block_settings))
                .branch(case![Command::EditPayment].endpoint(block_settings))
                .branch(case![Command::DeletePayment].endpoint(block_settings))
                .branch(case![Command::Settings].endpoint(handle_repeated_settings)),
        )
        .branch(
            case![State::SettingsTimeZone]
                .branch(case![Command::Start].endpoint(action_start))
                .branch(case![Command::Help].endpoint(action_help))
                .branch(case![Command::Cancel].endpoint(cancel_settings))
                .branch(case![Command::AddPayment].endpoint(block_settings))
                .branch(case![Command::Balances].endpoint(block_settings))
                .branch(case![Command::PayBack].endpoint(block_settings))
                .branch(case![Command::ViewPayments].endpoint(block_settings))
                .branch(case![Command::EditPayment].endpoint(block_settings))
                .branch(case![Command::DeletePayment].endpoint(block_settings))
                .branch(case![Command::Settings].endpoint(handle_repeated_settings)),
        )
        .branch(
            case![State::SettingsDefaultCurrencyMenu]
                .branch(case![Command::Start].endpoint(action_start))
                .branch(case![Command::Help].endpoint(action_help))
                .branch(case![Command::Cancel].endpoint(cancel_settings))
                .branch(case![Command::AddPayment].endpoint(block_settings))
                .branch(case![Command::Balances].endpoint(block_settings))
                .branch(case![Command::PayBack].endpoint(block_settings))
                .branch(case![Command::ViewPayments].endpoint(block_settings))
                .branch(case![Command::EditPayment].endpoint(block_settings))
                .branch(case![Command::DeletePayment].endpoint(block_settings))
                .branch(case![Command::Settings].endpoint(handle_repeated_settings)),
        )
        .branch(
            case![State::SettingsDefaultCurrency]
                .branch(case![Command::Start].endpoint(action_start))
                .branch(case![Command::Help].endpoint(action_help))
                .branch(case![Command::Cancel].endpoint(cancel_settings))
                .branch(case![Command::AddPayment].endpoint(block_settings))
                .branch(case![Command::Balances].endpoint(block_settings))
                .branch(case![Command::PayBack].endpoint(block_settings))
                .branch(case![Command::ViewPayments].endpoint(block_settings))
                .branch(case![Command::EditPayment].endpoint(block_settings))
                .branch(case![Command::DeletePayment].endpoint(block_settings))
                .branch(case![Command::Settings].endpoint(handle_repeated_settings)),
        )
        .branch(
            case![State::SettingsCurrencyConversion]
                .branch(case![Command::Start].endpoint(action_start))
                .branch(case![Command::Help].endpoint(action_help))
                .branch(case![Command::Cancel].endpoint(cancel_settings))
                .branch(case![Command::AddPayment].endpoint(block_settings))
                .branch(case![Command::Balances].endpoint(block_settings))
                .branch(case![Command::PayBack].endpoint(block_settings))
                .branch(case![Command::ViewPayments].endpoint(block_settings))
                .branch(case![Command::EditPayment].endpoint(block_settings))
                .branch(case![Command::DeletePayment].endpoint(block_settings))
                .branch(case![Command::Settings].endpoint(handle_repeated_settings)),
        );

    let message_handler = Update::filter_message()
        .branch(command_handler)
        .branch(case![State::AddDescription].endpoint(action_add_description))
        .branch(case![State::AddCreditor { payment }].endpoint(action_add_creditor))
        .branch(case![State::AddTotal { payment }].endpoint(action_add_total))
        .branch(
            case![State::AddDebt {
                payment,
                debts_format
            }]
            .endpoint(action_add_debt),
        )
        .branch(case![State::AddEdit { payment, edit }].endpoint(action_add_edit))
        .branch(case![State::PayBackCurrency].endpoint(action_pay_back_currency))
        .branch(case![State::PayBackDebts { currency }].endpoint(action_pay_back_debts))
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
        .branch(case![State::SettingsTimeZone].endpoint(action_settings_time_zone))
        .branch(case![State::SettingsDefaultCurrency].endpoint(action_settings_default_currency))
        .branch(case![State::AddDebtSelection { payment }].endpoint(callback_invalid_message))
        .branch(case![State::AddConfirm { payment }].endpoint(callback_invalid_message))
        .branch(case![State::AddEditDebtsMenu { payment }].endpoint(callback_invalid_message))
        .branch(case![State::AddEditMenu { payment }].endpoint(callback_invalid_message))
        .branch(case![State::PayBackCurrencyMenu].endpoint(callback_invalid_message))
        .branch(case![State::PayBackConfirm { payment }].endpoint(callback_invalid_message))
        .branch(
            case![State::SelectPayment {
                payments,
                page,
                function
            }]
            .endpoint(callback_invalid_message),
        )
        .branch(
            case![State::EditPayment {
                payment,
                edited_payment,
                payments,
                page
            }]
            .endpoint(callback_invalid_message),
        )
        .branch(
            case![State::EditPaymentDebtSelection {
                payment,
                edited_payment,
                payments,
                page
            }]
            .endpoint(callback_invalid_message),
        )
        .branch(
            case![State::DeletePayment {
                payment,
                payments,
                page
            }]
            .endpoint(callback_invalid_message),
        )
        .branch(case![State::SettingsMenu].endpoint(callback_invalid_message))
        .branch(case![State::SettingsTimeZoneMenu].endpoint(callback_invalid_message))
        .branch(case![State::SettingsDefaultCurrencyMenu].endpoint(callback_invalid_message))
        .branch(case![State::SettingsCurrencyConversion].endpoint(callback_invalid_message))
        .branch(case![State::Start].endpoint(invalid_state));

    let callback_query_handler = Update::filter_callback_query()
        .branch(case![State::AddDebtSelection { payment }].endpoint(action_add_debt_selection))
        .branch(case![State::AddConfirm { payment }].endpoint(action_add_confirm))
        .branch(case![State::AddEditDebtsMenu { payment }].endpoint(action_add_debt_selection))
        .branch(case![State::AddEditMenu { payment }].endpoint(action_add_edit_menu))
        .branch(case![State::PayBackCurrencyMenu].endpoint(action_pay_back_currency_menu))
        .branch(case![State::PayBackConfirm { payment }].endpoint(action_pay_back_confirm))
        .branch(case![State::ViewPayments { payments, page }].endpoint(action_view_more))
        .branch(
            case![State::SelectPayment {
                payments,
                page,
                function
            }]
            .endpoint(action_select_payment_number),
        )
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
            case![State::EditPaymentDebtSelection {
                payment,
                edited_payment,
                payments,
                page
            }]
            .endpoint(action_edit_payment_debts),
        )
        .branch(
            case![State::DeletePayment {
                payment,
                payments,
                page
            }]
            .endpoint(action_delete_payment_confirm),
        )
        .branch(case![State::SettingsMenu].endpoint(action_settings_menu))
        .branch(case![State::SettingsTimeZoneMenu].endpoint(action_time_zone_menu))
        .branch(case![State::SettingsDefaultCurrencyMenu].endpoint(action_default_currency_menu))
        .branch(
            case![State::SettingsCurrencyConversion].endpoint(action_settings_currency_conversion),
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
