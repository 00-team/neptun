use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
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
type HR = Result<(), Box<dyn Error + Send + Sync>>;

#[derive(Clone, Default, serde::Serialize, serde::Deserialize)]
pub enum State {
    #[default]
    Menu,
    AddRecord {
        id: i64,
    },
}

#[derive(BotCommands, Clone)]
#[command(rename_rule = "snake_case")]
pub enum Command {
    Start,
    Help,
    /// make a new record
    NewRecord,
}

#[derive(BotCommands, Clone)]
#[command(rename_rule = "snake_case")]
pub enum RecordCommand {
    /// finish sending messages for records
    EndRecord,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::from_path("./secrets.env").unwrap();
    pretty_env_logger::init();

    let pool: &'static SqlitePool = Box::leak(Box::new(SqlitePool::connect(&env::var("DATABASE_URL")?).await?));
    sqlx::migrate!().run(pool).await?;

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
        .branch(dptree::case![State::Menu].endpoint(menu))
        .branch(
            dptree::case![State::AddRecord { id }]
                .branch(
                    dptree::entry()
                        .filter_command::<RecordCommand>()
                        .endpoint(record_commands),
                )
                .endpoint(add_record),
        );

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![storage, pool])
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
    bot: Bot, dlg: Dialogue, pool: &SqlitePool, msg: Message, cmd: Command,
) -> HR {
    match cmd {
        Command::Start => {
            bot.send_message(msg.chat.id, "Welcome to the Neptun Bot.")
                .await?;
        }
        Command::Help => {
            bot.send_message(msg.chat.id, Command::descriptions().to_string())
                .await?;
        }
        Command::NewRecord => new_record(bot, dlg, pool, msg).await?,
    }

    Ok(())
}

async fn menu(bot: Bot, _dlg: Dialogue, msg: Message) -> HR {
    bot.send_message(
        msg.chat.id,
        "hi this is the menu of the bot\n/new_record.",
    )
    .await?;
    // dialogue.update(AddRecordState::Add).await?;

    Ok(())
}

async fn add_record(bot: Bot, id: i64, msg: Message) -> HR {
    bot.send_message(
        msg.chat.id,
        format!("adding new record to {}\nuse /end_record for finishing", id),
    )
    .await?;
    Ok(())
}

async fn new_record(
    bot: Bot, dlg: Dialogue, pool: &SqlitePool, msg: Message,
) -> HR {
    let record = models::Record {
        created_at: chrono::Local::now().timestamp(),
        slug: thread_rng()
            .sample_iter(&Alphanumeric)
            .take(16)
            .map(char::from)
            .collect(),
        ..Default::default()
    };

    let result = sqlx::query_as!(
        Record,
        "insert into records (slug, created_at, messages) values(?, ?, ?)",
        record.slug,
        record.created_at,
        record.messages
    )
    .execute(pool)
    .await?;

    let id = result.last_insert_rowid();

    bot.send_message(
        msg.chat.id,
        format!("send your messages\nyour record id is: {}", id),
    )
    .await?;

    dlg.update(State::AddRecord { id }).await?;

    Ok(())
}

async fn record_commands(
    bot: Bot, dlg: Dialogue, id: i64, msg: Message, _cmd: RecordCommand,
) -> HR {
    bot.send_message(
        msg.chat.id,
        format!("total messages: 69, ending: {}", id),
    )
    .await?;

    dlg.update(State::Menu).await?;

    Ok(())
}
