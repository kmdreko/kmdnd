use futures::TryStreamExt;
use mongodb::{bson, Database};

use crate::error::Error;

use super::{Item, ItemId};

const ITEMS: &str = "items";

pub async fn initialize(_db: &Database) -> Result<(), Error> {
    Ok(())
}

#[tracing::instrument(skip(db))]
pub async fn insert_item(db: &Database, item: &Item) -> Result<(), Error> {
    let doc = bson::to_document(item)?;
    db.collection(ITEMS).insert_one(doc, None).await?;

    Ok(())
}

#[tracing::instrument(skip(db))]
pub async fn fetch_items(db: &Database) -> Result<Vec<Item>, Error> {
    let items = db
        .collection(ITEMS)
        .find(bson::doc! {}, None)
        .await?
        .try_collect()
        .await?;

    Ok(items)
}

#[tracing::instrument(skip(db))]
pub async fn fetch_item_by_id(db: &Database, item_id: ItemId) -> Result<Option<Item>, Error> {
    let item = db
        .collection(ITEMS)
        .find_one(bson::doc! { "_id": item_id }, None)
        .await?;

    Ok(item)
}
