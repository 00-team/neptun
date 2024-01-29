use serde::{Deserialize, Serialize};
use sqlx::encode::IsNull;
use sqlx::sqlite::{SqliteArgumentValue, SqliteTypeInfo};
use sqlx::Sqlite;
use sqlx::{Encode, Type};
use teloxide::types::{ChatId, MessageId};
use std::borrow::Cow;
use std::vec::Vec;

#[derive(
    Clone, PartialEq, Eq, Debug, Serialize, Deserialize, sqlx::FromRow,
)]
pub struct Messages {
    pub cid: ChatId,
    pub ids: Vec<MessageId>,
}


#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Record {
    pub id: i64,
    pub slug: String,
    pub created_at: i64,
    pub messages: Messages,
    pub done: bool,
    pub count: i64,
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
        let result = serde_json::to_string(self).unwrap_or("{}".to_string());
        buf.push(SqliteArgumentValue::Text(Cow::Owned(result)));

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
        let result: Self = serde_json::from_str(&value).unwrap();
        result
    }
}
