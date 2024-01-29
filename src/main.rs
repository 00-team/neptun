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
    None,
    AddRecord {
        id: i64,
    },
}

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
pub enum Command {
    Start,
    Help,
    /// make a new record
    NewRecord,
    /// finish sending messages for records
    EndRecord,
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
            dptree::case![State::AddRecord { id }]
                .filter_command::<Command>()
                .endpoint(end_record),
        )
        .branch(
            dptree::entry()
                .filter_command::<Command>()
                .endpoint(handle_commands),
        )
        .branch(dptree::case![State::AddRecord { id }].endpoint(add_record));

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
    bot: Bot, dlg: Dialogue, msg: Message, cmd: Command,
) -> HR {
    match cmd {
        Command::Start => start(bot, dlg, msg).await?,
        Command::Help => {
            bot.send_message(msg.chat.id, Command::descriptions().to_string())
                .await?;
        }
        Command::NewRecord => new_record(bot, dlg, msg).await?,
        _ => (),
    }

    Ok(())
}

async fn start(bot: Bot, dlg: Dialogue, msg: Message) -> HR {
    bot.send_message(msg.chat.id, "Welcome to the Neptun Bot.")
        .await?;
    // dialogue.update(AddRecordState::Add).await?;

    Ok(())
}

async fn add_record(bot: Bot, dlg: Dialogue, id: i64, msg: Message) -> HR {
    bot.send_message(msg.chat.id, format!("adding new record to {}", id))
        .await?;
    // dialogue.update(State::End).await?;
    Ok(())
}

async fn new_record(bot: Bot, dlg: Dialogue, msg: Message) -> HR {
    bot.send_message(msg.chat.id, "new messages").await?;
    dlg.update(State::AddRecord { id: 12 }).await?;
    Ok(())
}

async fn end_record(
    bot: Bot, dlg: Dialogue, id: i64, msg: Message, cmd: Command,
) -> HR {
    match cmd {
        Command::EndRecord => {
            bot.send_message(
                msg.chat.id,
                format!("total messages: 69, ending: {}", id),
            )
            .await?;
            dlg.update(State::None).await?;
        }
        _ => (),
    }

    Ok(())
}
