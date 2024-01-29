use sqlx::SqlitePool;
use std::env;
use std::error::Error;
use std::sync::Arc;
use teloxide::dispatching::dialogue;
use teloxide::dispatching::dialogue::serializer::Json;
use teloxide::dispatching::dialogue::{ErasedStorage, SqliteStorage, Storage};
use teloxide::dispatching::{HandlerExt, UpdateFilterExt};
use teloxide::prelude::*;
use teloxide::utils::command::BotCommands;

mod config;
mod models;
use config::config;

type Dialogue = dialogue::Dialogue<State, ErasedStorage<State>>;
type HandlerResult = Result<(), Box<dyn Error + Send + Sync>>;

#[derive(Clone, Default, serde::Serialize, serde::Deserialize)]
pub enum State {
    #[default]
    Start,
    Menu,
    AddRecord,
    EndRecord,
}

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
pub enum Command {
    Help,
    /// random docs
    Start,
    Menu,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::from_path("./secrets.env").unwrap();
    pretty_env_logger::init();

    let pool = SqlitePool::connect(&env::var("DATABASE_URL")?).await?;
    sqlx::migrate!().run(&pool).await?;

    let bot = Bot::from_env();

    bot.send_message(config().dev, "Starting 🐧").await?;

    let storage: Arc<ErasedStorage<State>> =
        SqliteStorage::open(&env::var("TELOXIDE_STORAGE")?, Json)
            .await?
            .erase();

    let handler = Update::filter_message()
        .enter_dialogue::<Message, ErasedStorage<State>, State>()
        .branch(
            dptree::entry()
                .filter_command::<Command>()
                .endpoint(handle_commands),
        )
        .branch(dptree::case![State::Start].endpoint(start))
        .branch(dptree::case![State::Menu].endpoint(menu))
        .branch(dptree::case![State::AddRecord].endpoint(add_record))
        .branch(dptree::case![State::EndRecord].endpoint(end_record));

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![storage])
        .build()
        .dispatch()
        .await;

    // return Ok(());

    // teloxide::repl(bot, |bot: Bot, msg: Message| async move {
    //     bot.send_message(msg.chat.id, "hello from neptun! 🌩")
    //         .await?;
    //     Ok(())
    // })
    // .await;

    // let mut record = Record::default();

    // record.created_at = SystemTime::now()
    //     .duration_since(UNIX_EPOCH)
    //     .unwrap()
    //     .as_secs() as i64;
    //
    // record.slug = thread_rng()
    //     .sample_iter(&Alphanumeric)
    //     .take(16)
    //     .map(char::from)
    //     .collect();
    //
    // record.messages.ids.push(10);
    // record.messages.ids.push(12);

    // let result = sqlx::query_as!(
    //     Record,
    //     "insert into records (slug, created_at, messages) values(?, ?, ?)",
    //     record.slug,
    //     record.created_at,
    //     record.messages
    // )
    // .execute(&pool)
    // .await?;
    //
    // record.id = result.last_insert_rowid();
    //
    // println!("{:?}", record);

    // if let Err(e) = user.insert(&pool).await {
    //     println!("{:?}", e);
    // }

    // let new_record = sqlx::query_as!(Record, "select * from records where id = $1", 1)
    //     .fetch_one(&pool)
    //     .await?;
    //
    // println!("{:?}", new_record);

    Ok(())
}

async fn handle_commands(
    bot: Bot, dialogue: Dialogue, msg: Message, cmd: Command,
) -> HandlerResult {
    match cmd {
        Command::Start => {
            bot.send_message(msg.chat.id, "start command").await?;
        }
        Command::Menu => {
            bot.send_message(msg.chat.id, "start menu ").await?;
        }
        Command::Help => {
            bot.send_message(msg.chat.id, Command::descriptions().to_string())
                .await?;
        }
    }

    Ok(())
}

async fn start(bot: Bot, dialogue: Dialogue, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, "Welcome to the Neptun Bot.")
        .await?;
    // dialogue.update(AddRecordState::Add).await?;

    Ok(())
}

async fn menu(bot: Bot, dialogue: Dialogue, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, "hi this is the add!").await?;
    // dialogue.update(State::End).await?;
    Ok(())
}

async fn add_record(
    bot: Bot, dialogue: Dialogue, msg: Message,
) -> HandlerResult {
    bot.send_message(msg.chat.id, "hi this is the add!").await?;
    // dialogue.update(State::End).await?;
    Ok(())
}

async fn end_record(
    bot: Bot, dialogue: Dialogue, msg: Message,
) -> HandlerResult {
    bot.send_message(msg.chat.id, "hi this is the end!").await?;
    dialogue.reset().await?;
    Ok(())
}
