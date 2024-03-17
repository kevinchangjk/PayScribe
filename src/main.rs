mod bot;

use bot::handler;
use payscribe::bot::State;
use teloxide::{dispatching::dialogue::InMemStorage, prelude::*};

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    pretty_env_logger::init();
    log::info!("Starting PayScribe bot...");

    let bot = teloxide::Bot::from_env();

    log::info!("PayScribe bot started successfully!");

    Dispatcher::builder(bot, handler())
        .dependencies(dptree::deps![InMemStorage::<State>::new()])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}
