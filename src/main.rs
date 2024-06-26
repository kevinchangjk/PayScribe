use payscribe::bot::run_dispatcher;

#[tokio::main]
pub async fn main() {
    dotenv::dotenv().ok();
    pretty_env_logger::init();
    log::info!("Main - Starting PayScribe bot...");

    let bot = teloxide::Bot::from_env();

    log::info!("Main - PayScribe bot started successfully!");

    run_dispatcher(bot).await;
}
