use serenity::UserId;

use crate::prelude::*;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct UserDcEvent {
    pub metadata: UserDcEventMetadata,
    pub timestamp: mongodb::bson::DateTime,
}

impl UserDcEvent {
    pub fn new(
        guild_id: impl Into<serenity::GuildId>,
        user_id: impl Into<serenity::UserId>,
        event: UserEventType,
    ) -> Self {
        Self {
            metadata: UserDcEventMetadata {
                user_id: user_id.into(),
                guild_id: guild_id.into(),
                event,
            },
            timestamp: mongodb::bson::DateTime::from_chrono(chrono::Utc::now()),
        }
    }

    pub async fn user_is_active(
        user_id: impl Into<serenity::UserId>,
        guild_id: impl Into<serenity::GuildId>,
    ) -> Result<bool, mongodb::error::Error> {
        let col = Self::get_collection().await;

        let user_id: serenity::UserId = user_id.into();
        let guild_id: serenity::GuildId = guild_id.into();

        let filter = doc! {"metadata.user_id": user_id.to_string(), "metadata.guild_id": guild_id.to_string()};
        let sort = doc! {"timestamp": -1};

        let user = col.find(filter).sort(sort).limit(1).await?.next().await;

        match user {
            Some(user) => match user?.metadata.event {
                UserEventType::Left => Ok(false),
                UserEventType::Joined => Ok(true),
            },
            None => Ok(false),
        }
    }

    pub async fn active_users(
        guild_id: impl Into<serenity::GuildId>,
    ) -> Result<Vec<UserId>, Error> {
        let col = Self::get_collection().await;

        let guild_id: serenity::GuildId = guild_id.into();

        let mut users = col
            .aggregate(vec![
                doc! {
                    "$match": doc! {
                        "metadata.guild_id": guild_id.to_string(),
                    }
                },
                doc! {
                    "$sort": doc! {
                        "timestamp": -1
                    }
                },
                doc! {
                    "$group": doc! {
                        "_id": "$metadata.user_id",
                        "event": {
                            "$first": "$metadata.event"
                        }
                    }
                },
                doc! {
                    "$match": doc! {
                        "event": UserEventType::Joined
                    }
                },
            ])
            .await?;

        let mut out = vec![];

        while let Some(user) = users.next().await {
            out.push(user?.get_str("_id")?.parse::<u64>()?.into())
        }

        Ok(out)
    }

    pub async fn setup_collection() -> Result<(), mongodb::error::Error> {
        let db = Self::get_database().await;

        let options = mongodb::options::CreateCollectionOptions::builder()
            .timeseries(Some(
                mongodb::options::TimeseriesOptions::builder()
                    .time_field("timestamp".to_string())
                    .meta_field(Some("metadata".to_string()))
                    .granularity(Some(mongodb::options::TimeseriesGranularity::Seconds))
                    .build(),
            ))
            .build();

        let _ = db
            .create_collection(Self::COLLECTION)
            .with_options(options)
            .await;

        let index = mongodb::IndexModel::builder()
            .keys(doc! {"timestamp": 1})
            .options(
                mongodb::options::IndexOptions::builder()
                    .expire_after(std::time::Duration::from_secs(604800 * 2))
                    .partial_filter_expression(Some(doc! {
                        "metadata": {"$exists": true}
                    }))
                    .build(),
            )
            .build();

        let _ = Self::get_collection().await.create_index(index).await;
        Ok(())
    }
}

impl MongoCrud for UserDcEvent {
    const COLLECTION: &'static str = "user_event";
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct UserDcEventMetadata {
    pub guild_id: serenity::GuildId,
    pub user_id: serenity::UserId,
    pub event: UserEventType,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum UserEventType {
    Joined,
    Left,
}

impl From<UserEventType> for mongodb::bson::Bson {
    fn from(value: UserEventType) -> Self {
        match value {
            UserEventType::Left => "Left".into(),
            UserEventType::Joined => "Joined".into(),
        }
    }
}
