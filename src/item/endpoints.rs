use actix_web::get;
use actix_web::web::{Data, Json, Path};
use mongodb::Database;
use serde::Serialize;

use crate::error::Error;

use super::{db, Item, ItemId, ItemType};

#[derive(Clone, Debug, Serialize)]
pub struct ItemBody {
    pub id: ItemId,
    pub name: String,
    pub weight: i32,
    pub value: i32,
    pub item_type: ItemType,
}

impl ItemBody {
    pub fn render(item: Item) -> ItemBody {
        ItemBody {
            id: item.id,
            name: item.name,
            weight: item.weight,
            value: item.value,
            item_type: item.item_type,
        }
    }
}

#[get("/items")]
#[tracing::instrument(skip(db))]
async fn get_items(db: Data<Database>) -> Result<Json<Vec<ItemBody>>, Error> {
    let items = db::fetch_items(&db).await?;

    let body = items
        .into_iter()
        .map(|item| ItemBody::render(item))
        .collect();

    Ok(Json(body))
}

#[get("/items/{item_id}")]
#[tracing::instrument(skip(db))]
async fn get_item_by_id(db: Data<Database>, params: Path<ItemId>) -> Result<Json<ItemBody>, Error> {
    let item_id = params.into_inner();

    let item = db::fetch_item_by_id(&db, item_id)
        .await?
        .ok_or(Error::ItemDoesNotExist { item_id })?;

    Ok(Json(ItemBody::render(item)))
}
