use std::error::Error;
use std::sync::Arc;
use teloxide::dispatching::dialogue::serializer::Json;
use teloxide::dispatching::{HandlerExt, UpdateFilterExt};
use teloxide::prelude::*;

// use rand::distributions::Alphanumeric;
// use rand::{thread_rng, Rng};
use sqlx::SqlitePool;
// use std::convert::From;
use std::env;
// use std::string::String;
// use std::time::{SystemTime, UNIX_EPOCH};

mod config;
mod models;
// use models::Record;
use config::config;

use teloxide::dispatching::dialogue::{ErasedStorage, SqliteStorage, Storage};

type AddRecordDialogue = Dialogue<AddRecordState, ErasedStorage<AddRecordState>>;
type HandlerResult = Result<(), Box<dyn Error + Send + Sync>>;
type ARStorage = Arc<ErasedStorage<AddRecordState>>;

#[derive(Clone, Default, serde::Serialize, serde::Deserialize)]
pub enum AddRecordState {
    #[default]
    Start,
    Add,
    End,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::from_path("./secrets.env").unwrap();
    pretty_env_logger::init();

    let pool = SqlitePool::connect(&env::var("DATABASE_URL")?).await?;
    sqlx::migrate!().run(&pool).await?;

    let bot = Bot::from_env();

    bot.send_message(config().dev, "Starting 🐧").await?;

    let storage: ARStorage = SqliteStorage::open(&env::var("TELOXIDE_STORAGE")?, Json)
        .await?
        .erase();

    let handler = Update::filter_message()
        .enter_dialogue::<Message, ErasedStorage<AddRecordState>, AddRecordState>()
        .branch(dptree::case![AddRecordState::Start].endpoint(start))
        .branch(dptree::case![AddRecordState::Add].endpoint(add))
        .branch(dptree::case![AddRecordState::End].endpoint(end));

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

async fn start(bot: Bot, dialogue: AddRecordDialogue, msg: Message) -> HandlerResult {
    log::info!("config: {:#?}", config());
    // log::info!("users message: {:#?}", msg);

    bot.send_message(msg.chat.id, "hi this is the start!")
        .await?;
    // dialogue.update(AddRecordState::Add).await?;

    Ok(())
}

async fn add(bot: Bot, dialogue: AddRecordDialogue, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, "hi this is the add!").await?;
    dialogue.update(AddRecordState::End).await?;
    Ok(())
}

async fn end(bot: Bot, dialogue: AddRecordDialogue, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, "hi this is the end!").await?;
    dialogue.reset().await?;
    Ok(())
}
