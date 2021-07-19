use crate::database::Database;
use crate::error::Error;

use super::{Item, ItemId};

#[tracing::instrument(skip(db))]
pub async fn get_items(db: &dyn Database) -> Result<Vec<Item>, Error> {
    let items = db.items().fetch_items().await?;

    Ok(items)
}

#[tracing::instrument(skip(db))]
pub async fn get_item_by_id(db: &dyn Database, item_id: ItemId) -> Result<Option<Item>, Error> {
    let item = db.items().fetch_item_by_id(item_id).await?;

    Ok(item)
}

#[tracing::instrument(skip(db))]
pub async fn expect_item_by_id(db: &dyn Database, item_id: ItemId) -> Result<Item, Error> {
    let item = db
        .items()
        .fetch_item_by_id(item_id)
        .await?
        .ok_or(Error::ItemExpected { item_id })?;

    Ok(item)
}
