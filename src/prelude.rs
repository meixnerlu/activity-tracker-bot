use std::env;

use async_once_cell::OnceCell;
use moka::sync::Cache;
pub use mongodb::bson::doc;
use mongodb::Database;

pub static STATE: OnceCell<State> = OnceCell::new();
pub use crate::utils::*;
pub use poise::command;
pub use poise::serenity_prelude::{self as serenity, futures::StreamExt, CacheHttp, Mentionable};
pub use serde::{de::DeserializeOwned, Deserialize, Serialize};

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

static DATABASE: &str = "activity-tracker";

pub struct Data {}

impl Data {
    pub async fn new() -> Self {
        Self {}
    }
}

pub struct State {
    db: Database,
    guild_cache: Cache<serenity::GuildId, Option<serenity::RoleId>>,
}

impl State {
    pub async fn new() -> Self {
        let url = env::var("MONGODB").expect("MONGODB");
        let client = mongodb::Client::with_uri_str(url).await.unwrap();

        Self {
            db: client.database(DATABASE),
            guild_cache: Cache::new(10_000),
        }
    }

    pub async fn global() -> &'static Self {
        STATE.get_or_init(Self::new()).await
    }

    pub fn db(&self) -> &Database {
        &self.db
    }

    pub fn guild_cache(&self) -> &Cache<serenity::GuildId, Option<serenity::RoleId>> {
        &self.guild_cache
    }
}
