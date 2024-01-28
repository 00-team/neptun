/*
use teloxide::prelude::*;

#[tokio::main]
async fn main() {
    dotenvy::from_path("./secrets.env").unwrap();
    pretty_env_logger::init();
    log::info!("Starting throw dice bot...");

    let bot = Bot::from_env();


    teloxide::repl(bot, |bot: Bot, msg: Message| async move {
        bot.send_message(msg.chat.id, "hello from neptun! 🌩").await?;
        Ok(())
    })
    .await;
}
*/

use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use sqlx::encode::IsNull;
use sqlx::sqlite::SqliteTypeInfo;
use sqlx::{Encode, Type};
use sqlx::{Sqlite, SqlitePool};
use std::convert::From;
use std::env;
use std::string::String;
use std::time::{SystemTime, UNIX_EPOCH};
use std::vec::Vec;

#[derive(Clone, PartialEq, Eq, Debug, Default, Serialize, Deserialize, sqlx::FromRow)]
struct Messages {
    ids: Vec<i64>,
}

#[derive(Debug, Default, Serialize, Deserialize, sqlx::FromRow)]
struct Record {
    id: i64,
    slug: String,
    created_at: i64,
    messages: Messages,
    done: bool,
    count: i64,
}

// impl<'r> Decode<'r, Sqlite> for Messages
// where
//     &'r str: Decode<'r, Sqlite>,
// {
//     fn decode(
//         // value: <DB as sqlx::database::HasValueRef<'r>>::ValueRef,
//         value: SqliteValueRef
//     ) -> Result<Self, sqlx::error::BoxDynError> {
//         Ok(Self {
//             ids: vec![4, 4, 44],
//         })
//     }
// }

impl<'q> Encode<'q, Sqlite> for Messages {
    fn encode_by_ref(
        &self,
        buf: &mut <Sqlite as sqlx::database::HasArguments<'q>>::ArgumentBuffer,
    ) -> IsNull {
        buf.push(sqlx::sqlite::SqliteArgumentValue::Text(
            std::borrow::Cow::Owned("[3, 4]".to_owned()),
        ));

        IsNull::No
    }
}

impl Type<Sqlite> for Messages {
    fn type_info() -> SqliteTypeInfo {
        <&str as Type<Sqlite>>::type_info()
    }
}

impl From<String> for Messages {
    fn from(value: String) -> Self {
        Self { ids: vec![8, 0] }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::from_path("./secrets.env").unwrap();
    let pool = SqlitePool::connect(&env::var("DATABASE_URL")?).await?;

    sqlx::migrate!().run(&pool).await?;

    let mut record = Record::default();

    record.created_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    record.slug = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(16)
        .map(char::from)
        .collect();

    record.messages.ids.push(10);
    record.messages.ids.push(12);

    let result = sqlx::query_as!(
        Record,
        "insert into records (slug, created_at, messages) values(?, ?, ?)",
        record.slug,
        record.created_at,
        record.messages
    )
    .execute(&pool)
    .await?;

    record.id = result.last_insert_rowid();

    println!("{:?}", record);

    // if let Err(e) = user.insert(&pool).await {
    //     println!("{:?}", e);
    // }

    let new_record = sqlx::query_as!(Record, "select * from records where id = $1", record.id)
        .fetch_one(&pool)
        .await?;

    println!("{:?}", new_record);

    Ok(())
}
