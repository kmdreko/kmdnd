use futures::TryStreamExt;
use mongodb::{bson, Database};

use crate::database::ItemStore;
use crate::error::Error;

use super::{Item, ItemId};

const ITEMS: &str = "items";

pub async fn initialize(_db: &Database) -> Result<(), Error> {
    Ok(())
}

#[tracing::instrument(skip(db))]
pub async fn insert_item(db: &ItemStore, item: &Item) -> Result<(), Error> {
    db.insert_one(item, None).await?;

    Ok(())
}

#[tracing::instrument(skip(db))]
pub async fn fetch_items(db: &ItemStore) -> Result<Vec<Item>, Error> {
    let items = db.find(bson::doc! {}, None).await?.try_collect().await?;

    Ok(items)
}

#[tracing::instrument(skip(db))]
pub async fn fetch_item_by_id(db: &ItemStore, item_id: ItemId) -> Result<Option<Item>, Error> {
    let item = db.find_one(bson::doc! { "_id": item_id }, None).await?;

    Ok(item)
}
