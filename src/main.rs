
use teloxide::prelude::*;

#[tokio::main]
async fn main() {
    dotenvy::from_path("./secrets.env").unwrap();
    pretty_env_logger::init();
    log::info!("Starting throw dice bot...");

    let bot = Bot::from_env();

    teloxide::repl(bot, |bot: Bot, msg: Message| async move {
        bot.send_message(msg.chat.id, "hello from neptun").await?;
        Ok(())
    })
    .await;
}

