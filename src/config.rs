use std::{env, sync::OnceLock};

use teloxide::types::UserId;

#[derive(Debug)]
pub struct Config {
    pub dev: UserId,
    pub admins: Vec<UserId>,
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

        Config { dev, admins }
    })
}
