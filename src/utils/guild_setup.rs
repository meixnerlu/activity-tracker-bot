use crate::prelude::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuildSetup {
    pub guild_id: serenity::GuildId,
    pub channel_id: serenity::ChannelId,
    pub role_to_watch: Option<serenity::RoleId>,
    pub leaderboard_message: serenity::MessageId,
}

impl GuildSetup {
    pub fn new(
        guild_id: impl Into<serenity::GuildId>,
        channel_id: impl Into<serenity::ChannelId>,
        role_to_watch: Option<serenity::RoleId>,
        leaderboard_message: impl Into<serenity::MessageId>,
    ) -> Self {
        Self {
            guild_id: guild_id.into(),
            channel_id: channel_id.into(),
            role_to_watch,
            leaderboard_message: leaderboard_message.into(),
        }
    }

    pub async fn remove(guild_id: impl Into<serenity::GuildId>) -> Result<(), Error> {
        let guild_id = guild_id.into();
        let state = State::global().await;

        Self::delete(doc! {"guild_id": guild_id.to_string()}).await?;

        state.guild_cache().remove(&guild_id);

        Ok(())
    }

    pub async fn get_guilds() -> Result<Vec<Self>, Error> {
        let cache = State::global().await.guild_cache();

        let mut cursor = Self::get_collection().await.find(doc! {}).await?;

        let mut out = vec![];

        while let Some(guild) = cursor.next().await {
            let guild = guild?;

            out.push(guild.clone());
            cache.insert(guild.guild_id, guild.role_to_watch);
        }

        Ok(out)
    }

    pub async fn guild_exists(guild_id: impl Into<serenity::GuildId>) -> Result<bool, Error> {
        let guild_id = guild_id.into();
        let state = State::global().await;
        match state.guild_cache().get(&guild_id) {
            Some(_) => Ok(true),
            None => {
                let setup = GuildSetup::get(doc! {"guild_id": guild_id.to_string()}).await?;
                match setup {
                    Some(setup) => {
                        state
                            .guild_cache()
                            .insert(setup.guild_id, setup.role_to_watch);
                        Ok(false)
                    }
                    None => Ok(false),
                }
            }
        }
    }

    pub async fn get_role(
        guild_id: impl Into<serenity::GuildId>,
    ) -> Result<Option<serenity::RoleId>, Error> {
        let guild_id = guild_id.into();
        let state = State::global().await;
        match state.guild_cache().get(&guild_id) {
            Some(data) => Ok(data),
            None => {
                let setup = GuildSetup::get(doc! {"guild_id": guild_id.to_string()}).await?;
                match setup {
                    Some(setup) => {
                        state
                            .guild_cache()
                            .insert(setup.guild_id, setup.role_to_watch);
                        Ok(setup.role_to_watch)
                    }
                    None => Ok(None),
                }
            }
        }
    }

    pub async fn setup_collection() -> Result<(), mongodb::error::Error> {
        let db = Self::get_database().await;

        let options = mongodb::options::CreateCollectionOptions::default();

        let _ = db
            .create_collection(Self::COLLECTION)
            .with_options(options)
            .await;

        let index = mongodb::IndexModel::builder()
            .keys(doc! {"guild_id": 1})
            .options(
                mongodb::options::IndexOptions::builder()
                    .unique(true)
                    .build(),
            )
            .build();

        let _ = Self::get_collection().await.create_index(index).await;
        Ok(())
    }
}

impl MongoCrud for GuildSetup {
    const COLLECTION: &'static str = "guild_setup";

    async fn insert(&self) -> Result<(), mongodb::error::Error> {
        Self::get_collection().await.insert_one(self).await?;

        let cache = State::global().await.guild_cache();

        cache.insert(self.guild_id, self.role_to_watch);

        Ok(())
    }
}
