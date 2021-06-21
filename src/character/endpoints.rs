use actix_web::web::{Data, Json, Path};
use actix_web::{get, post};
use chrono::{DateTime, Utc};
use futures::{stream, StreamExt, TryStreamExt};
use mongodb::Database;
use serde::{Deserialize, Serialize};

use crate::campaign::{self, CampaignId};
use crate::error::Error;
use crate::item::{self, ItemBody};

use super::{db, Character, CharacterId, CharacterOwner, CharacterStats};

#[derive(Clone, Debug, Deserialize)]
pub struct CreateCharacterBody {
    pub name: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct CharacterBody {
    pub id: CharacterId,
    pub owner: CharacterOwner,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
    pub stats: CharacterStats,
    pub equipment: Vec<ItemWithQuantityBody>,
}

impl CharacterBody {
    pub async fn render(db: &Database, character: Character) -> Result<CharacterBody, Error> {
        let mut equipment = vec![];
        for entry in character.equipment {
            let item = item::db::fetch_item_by_id(db, entry.item_id).await?.ok_or(
                Error::ItemDoesNotExist {
                    item_id: entry.item_id,
                },
            )?;
            let body = ItemBody::render(item);
            equipment.push(ItemWithQuantityBody {
                quantity: entry.quantity,
                item: body,
            });
        }

        Ok(CharacterBody {
            id: character.id,
            owner: character.owner,
            name: character.name,
            created_at: character.created_at,
            modified_at: character.modified_at,
            stats: character.stats,
            equipment: equipment,
        })
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct ItemWithQuantityBody {
    pub quantity: i32,
    pub item: ItemBody,
}

#[post("/campaigns/{campaign_id}/characters")]
#[tracing::instrument(skip(db))]
async fn create_character_in_campaign(
    db: Data<Database>,
    params: Path<CampaignId>,
    body: Json<CreateCharacterBody>,
) -> Result<Json<CharacterBody>, Error> {
    let campaign_id = params.into_inner();
    let body = body.into_inner();

    campaign::db::fetch_campaign_by_id(&db, campaign_id)
        .await?
        .ok_or(Error::CampaignDoesNotExist { campaign_id })?;

    let now = Utc::now();
    let character = Character {
        id: CharacterId::new(),
        owner: CharacterOwner::Campaign(campaign_id),
        name: body.name,
        created_at: now,
        modified_at: now,
        stats: Default::default(),
        equipment: vec![],
    };

    db::insert_character(&db, &character).await?;

    Ok(Json(CharacterBody::render(&db, character).await?))
}

#[get("/campaigns/{campaign_id}/characters")]
#[tracing::instrument(skip(db))]
async fn get_characters_in_campaign(
    db: Data<Database>,
    params: Path<CampaignId>,
) -> Result<Json<Vec<CharacterBody>>, Error> {
    let campaign_id = params.into_inner();

    campaign::db::fetch_campaign_by_id(&db, campaign_id)
        .await?
        .ok_or(Error::CampaignDoesNotExist { campaign_id })?;

    let characters = db::fetch_characters_by_campaign(&db, campaign_id).await?;

    let body = stream::iter(characters)
        .then(|character| CharacterBody::render(&db, character))
        .try_collect()
        .await?;

    Ok(Json(body))
}

#[get("/campaigns/{campaign_id}/characters/{character_id}")]
#[tracing::instrument(skip(db))]
async fn get_character_in_campaign_by_id(
    db: Data<Database>,
    params: Path<(CampaignId, CharacterId)>,
) -> Result<Json<CharacterBody>, Error> {
    let (campaign_id, character_id) = params.into_inner();

    campaign::db::fetch_campaign_by_id(&db, campaign_id)
        .await?
        .ok_or(Error::CampaignDoesNotExist { campaign_id })?;

    let character = db::fetch_character_by_campaign_and_id(&db, campaign_id, character_id)
        .await?
        .ok_or(Error::CharacterDoesNotExistInCampaign {
            campaign_id,
            character_id,
        })?;

    Ok(Json(CharacterBody::render(&db, character).await?))
}
