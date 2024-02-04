use indoc::{formatdoc, indoc};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use reqwest;
use sqlx::SqlitePool;
use std::env;
use std::error::Error;
use std::sync::Arc;
use teloxide::dispatching::dialogue;
use teloxide::dispatching::dialogue::serializer::Json;
use teloxide::dispatching::dialogue::{ErasedStorage, SqliteStorage, Storage};
use teloxide::dispatching::{HandlerExt, UpdateFilterExt};
use teloxide::prelude::*;
use teloxide::types::ParseMode::MarkdownV2;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};
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
    Start(String),
    Help,
    /// make a new record
    NewRecord,
    /// get a record by id
    GetRecord {
        id: i64,
    },
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

    let pool: &'static SqlitePool = Box::leak(Box::new(
        SqlitePool::connect(&env::var("DATABASE_URL")?).await?,
    ));
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

    Ok(())
}

async fn handle_commands(
    bot: Bot, dlg: Dialogue, pool: &SqlitePool, msg: Message, cmd: Command,
) -> HR {
    match cmd {
        Command::Start(arg) => {
            let arg = parse_start_args(&arg);
            match arg {
                StartArg::Record {id, slug: _} => {
                    get_record(bot, pool, id, msg).await?;
                },
                StartArg::None => {
                    bot.send_message(msg.chat.id, "Welcome to the Neptun Bot.").await?;
                }
            }
        }
        Command::Help => {
            bot.send_message(msg.chat.id, Command::descriptions().to_string())
                .await?;
        }
        Command::NewRecord => new_record(bot, dlg, pool, msg).await?,
        Command::GetRecord { id } => get_record(bot, pool, id, msg).await?,
    }

    Ok(())
}

enum StartArg {
    Record { id: i64, slug: String },
    None,
}

fn parse_start_args(arg: &str) -> StartArg {
    let mut value = arg.split("-");
    match value.nth(0) {
        None => StartArg::None,
        Some(key) => match key {
            "record" => {
                if let Some(id) = value.nth(1) {
                    if let Ok(id) = id.parse::<i64>() {
                        if let Some(slug) = value.nth(2) {
                            return StartArg::Record {
                                id,
                                slug: slug.to_owned(),
                            };
                        }
                    }
                }

                StartArg::None
            }
            _ => StartArg::None,
        },
    }

    // if arg.starts_with("record-") {
    //                 if let Some(sid) = arg.split("-").nth(1) {
    //                     if let Ok(rid) = sid.parse::<i64>() {
    //                         return get_record(bot, pool, rid, msg).await;
    //                     }
    //                 }
    //             }
}

async fn menu(bot: Bot, _dlg: Dialogue, msg: Message) -> HR {
    bot.send_message(
        msg.chat.id,
        indoc! {"
            hi this is the menu of the bot
            /new_record
        "},
    )
    .await?;
    // dialogue.update(AddRecordState::Add).await?;

    Ok(())
}

async fn get_record(bot: Bot, pool: &SqlitePool, id: i64, msg: Message) -> HR {
    let result = sqlx::query_as! {
        models::Record,
        "select * from records where id = ? and done = true", id,
    }
    .fetch_one(pool)
    .await;

    match result {
        Err(_) => {
            bot.send_message(
                msg.chat.id,
                format!("<Record {} /> was not found!", id),
            )
            .await?;
        }
        Ok(r) => {
            for mid in r.messages.ids {
                let mut x = bot.copy_message(msg.chat.id, r.messages.cid, mid);

                let mut cap = x.caption.clone().unwrap_or(String::new());
                cap.push_str("\nsome ad? 🥗");
                x.caption = Some(cap);

                x.send().await?;
                // bot.forward_message(msg.chat.id, r.messages.cid, mid).await?;
            }

            bot.send_message(
                msg.chat.id,
                format!("total messages: {}", r.count),
            )
            .await?;
        }
    }

    Ok(())
}

async fn add_record(
    bot: Bot, pool: &SqlitePool, dlg: Dialogue, id: i64, msg: Message,
) -> HR {
    let result = sqlx::query_as! {
        models::Record,
        "select * from records where id = ? and done = false", id,
    }
    .fetch_one(pool)
    .await;

    match result {
        Err(_) => dlg.update(State::Menu).await?,
        Ok(mut r) => {
            r.messages.ids.push(msg.id);
            r.count += 1;

            sqlx::query_as! {
                models::Record,
                "update records set messages = ?, count = ?
                where id = ? and done = false",
                r.messages, r.count, id
            }
            .execute(pool)
            .await?;
        }
    }

    bot.send_message(
        msg.chat.id,
        formatdoc! {"
            added <Message {} /> to <Record {} />
            use /end_record to finish", msg.id, id
        },
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
        id: 0,
        count: 0,
        done: false,
        messages: models::Messages { cid: msg.chat.id, ids: Vec::new() },
    };

    let result = sqlx::query_as! {
        Record,
        "insert into records (slug, created_at, messages) values(?, ?, ?)",
        record.slug, record.created_at, record.messages
    }
    .execute(pool)
    .await?;

    let id = result.last_insert_rowid();

    bot.send_message(
        msg.chat.id,
        formatdoc! {"
            send your messages
            your record id is: {}", id
        },
    )
    .await?;

    dlg.update(State::AddRecord { id }).await?;

    Ok(())
}

async fn record_commands(
    bot: Bot, dlg: Dialogue, pool: &SqlitePool, id: i64, msg: Message,
    _cmd: RecordCommand,
) -> HR {
    let result = sqlx::query_as!(
        models::Record,
        "select * from records where id = ? and done = false",
        id,
    )
    .fetch_one(pool)
    .await;

    match result {
        Err(_) => {
            bot.send_message(msg.chat.id, "record was not found!").await?;
        }
        Ok(r) => {
            let url = reqwest::Url::parse(&format!(
                "tg://@{}?start=record-{}-{}",
                config().bot_username,
                r.id,
                r.slug
            ))?;
            let keyboard: Vec<Vec<InlineKeyboardButton>> =
                vec![vec![InlineKeyboardButton::url("Get Record", url)]];

            bot.send_message(
                msg.chat.id,
                formatdoc! {"
                    total messages: {}
                    id: {}
                    get the record like `/get_record {}`",
                    r.count, r.id, r.id
                },
            )
            .parse_mode(MarkdownV2)
            .reply_markup(InlineKeyboardMarkup::new(keyboard))
            .await?;
        }
    }

    sqlx::query_as!(
        models::Record,
        "update records set done = true where id = ? and done = false",
        id
    )
    .execute(pool)
    .await?;

    dlg.update(State::Menu).await?;

    Ok(())
}
