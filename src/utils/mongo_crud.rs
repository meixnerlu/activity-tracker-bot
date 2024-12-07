use crate::prelude::*;

pub trait MongoCrud
where
    Self: Serialize + DeserializeOwned + Send + Sync,
{
    const COLLECTION: &'static str;

    async fn insert(&self) -> Result<(), mongodb::error::Error> {
        Self::get_collection()
            .await
            .insert_one(self)
            .await
            .map(|_| ())
    }

    async fn get(filter: mongodb::bson::Document) -> Result<Option<Self>, mongodb::error::Error> {
        Self::get_collection().await.find_one(filter).await
    }

    async fn change(
        filter: mongodb::bson::Document,
        change: mongodb::bson::Document,
    ) -> Result<(), mongodb::error::Error> {
        Self::get_collection()
            .await
            .update_many(filter, change)
            .await
            .map(|_| ())
    }

    async fn delete(filter: mongodb::bson::Document) -> Result<(), mongodb::error::Error> {
        Self::get_collection()
            .await
            .delete_one(filter)
            .await
            .map(|_| ())
    }

    async fn get_collection() -> mongodb::Collection<Self> {
        Self::get_database()
            .await
            .collection::<Self>(Self::COLLECTION)
    }

    async fn get_database() -> &'static mongodb::Database {
        State::global().await.db()
    }
}
