use async_trait::async_trait;
use futures::TryStreamExt;
use mongodb::{bson, Database};

use crate::database::MongoItemStore;
use crate::error::Error;

use super::{Item, ItemId};

const ITEMS: &str = "items";

#[async_trait]
pub trait ItemStore {
    async fn insert_item(&self, item: &Item) -> Result<(), Error>;

    async fn fetch_items(&self) -> Result<Vec<Item>, Error>;

    async fn fetch_item_by_id(&self, item_id: ItemId) -> Result<Option<Item>, Error>;
}

pub async fn initialize(_db: &Database) -> Result<(), Error> {
    Ok(())
}

#[async_trait]
impl ItemStore for MongoItemStore {
    #[tracing::instrument(skip(self))]
    async fn insert_item(&self, item: &Item) -> Result<(), Error> {
        self.insert_one(item, None).await?;

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn fetch_items(&self) -> Result<Vec<Item>, Error> {
        let items = self.find(bson::doc! {}, None).await?.try_collect().await?;

        Ok(items)
    }

    #[tracing::instrument(skip(self))]
    async fn fetch_item_by_id(&self, item_id: ItemId) -> Result<Option<Item>, Error> {
        let item = self.find_one(bson::doc! { "_id": item_id }, None).await?;

        Ok(item)
    }
}
