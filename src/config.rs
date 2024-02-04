use core::panic;
use std::{env, sync::OnceLock};

use teloxide::types::UserId;

#[derive(Debug)]
pub struct Config {
    pub dev: UserId,
    pub admins: Vec<UserId>,
    pub bot_username: String
}

pub fn config() -> &'static Config {
    static STATE: OnceLock<Config> = OnceLock::new();
    STATE.get_or_init(|| {
        let dev = UserId(
            env::var("TELOXIDE_DEVELOPER")
                .unwrap()
                .parse::<u64>()
                .unwrap(),
        );

        let mut admins: Vec<UserId> =
            serde_json::from_str(&env::var("TELOXIDE_ADMINS").unwrap())
                .unwrap();
        admins.push(dev);

        let bot_username = env::var("TELOXIDE_BOT_USERNAME").unwrap();
        if bot_username.starts_with("@") {
            panic!("bot_username must NOT start with @");
        }

        Config { dev, admins, bot_username }
    })
}
