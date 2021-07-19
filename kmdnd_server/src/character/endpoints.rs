use actix_web::web::{Data, Json, Path};
use actix_web::{get, post};
use chrono::{DateTime, Utc};
use futures::{stream, StreamExt, TryStreamExt};
use serde::{Deserialize, Serialize};

use crate::campaign::{self, CampaignId};
use crate::database::Database;
use crate::error::Error;
use crate::item::ItemBody;
use crate::operation::RollType;

use super::{
    manager, Character, CharacterId, CharacterOwner, CharacterStats, Position, RollModifier,
};

#[derive(Clone, Debug, Deserialize)]
pub struct CreateCharacterBody {
    pub name: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CharacterBody {
    pub id: CharacterId,
    pub owner: CharacterOwner,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
    pub stats: CharacterStats,
    pub equipment: Vec<ItemWithQuantityBody>,
    pub position: Option<Position>,
    pub current_hit_points: i32,
    pub maximum_hit_points: i32,
}

impl CharacterBody {
    pub async fn render(db: &dyn Database, character: Character) -> Result<CharacterBody, Error> {
        let mut equipment = vec![];
        for entry in character.equipment {
            let item = db.items().fetch_item_by_id(entry.item_id).await?.ok_or(
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
            equipment,
            position: character.position,
            current_hit_points: character.current_hit_points,
            maximum_hit_points: character.maximum_hit_points,
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ItemWithQuantityBody {
    pub quantity: i32,
    pub item: ItemBody,
}

#[derive(Clone, Debug, Serialize)]
pub struct RollStatsBody {
    modifier: RollModifier,
}

#[post("/campaigns/{campaign_id}/characters")]
#[tracing::instrument(skip(db))]
async fn create_character_in_campaign(
    db: Data<Box<dyn Database>>,
    params: Path<CampaignId>,
    body: Json<CreateCharacterBody>,
) -> Result<Json<CharacterBody>, Error> {
    let campaign_id = params.into_inner();
    let campaign = campaign::manager::get_campaign_by_id(&***db, campaign_id)
        .await?
        .ok_or(Error::CampaignNotFound { campaign_id })?;
    let body = body.into_inner();

    let character = manager::create_character_in_campaign(&***db, &campaign, body.name).await?;

    Ok(Json(CharacterBody::render(&***db, character).await?))
}

#[get("/campaigns/{campaign_id}/characters")]
#[tracing::instrument(skip(db))]
async fn get_characters_in_campaign(
    db: Data<Box<dyn Database>>,
    params: Path<CampaignId>,
) -> Result<Json<Vec<CharacterBody>>, Error> {
    let campaign_id = params.into_inner();
    let campaign = campaign::manager::get_campaign_by_id(&***db, campaign_id)
        .await?
        .ok_or(Error::CampaignNotFound { campaign_id })?;

    let characters = manager::get_characters_in_campaign(&***db, &campaign).await?;

    let body = stream::iter(characters)
        .then(|character| CharacterBody::render(&***db, character))
        .try_collect()
        .await?;

    Ok(Json(body))
}

#[get("/campaigns/{campaign_id}/characters/{character_id}")]
#[tracing::instrument(skip(db))]
async fn get_character_in_campaign_by_id(
    db: Data<Box<dyn Database>>,
    params: Path<(CampaignId, CharacterId)>,
) -> Result<Json<CharacterBody>, Error> {
    let (campaign_id, character_id) = params.into_inner();
    let campaign = campaign::manager::get_campaign_by_id(&***db, campaign_id)
        .await?
        .ok_or(Error::CampaignNotFound { campaign_id })?;
    let character = manager::get_character_in_campaign_by_id(&***db, &campaign, character_id)
        .await?
        .ok_or(Error::CharacterNotFoundInCampaign {
            campaign_id: campaign.id,
            character_id,
        })?;

    Ok(Json(CharacterBody::render(&***db, character).await?))
}

#[get("/campaigns/{campaign_id}/characters/{character_id}/roll/{roll_type}")]
#[tracing::instrument(skip(db))]
async fn get_character_roll_stats(
    db: Data<Box<dyn Database>>,
    params: Path<(CampaignId, CharacterId, RollType)>,
) -> Result<Json<RollStatsBody>, Error> {
    let (campaign_id, character_id, roll_type) = params.into_inner();
    let campaign = campaign::manager::get_campaign_by_id(&***db, campaign_id)
        .await?
        .ok_or(Error::CampaignNotFound { campaign_id })?;
    let character = manager::get_character_in_campaign_by_id(&***db, &campaign, character_id)
        .await?
        .ok_or(Error::CharacterNotFoundInCampaign {
            campaign_id: campaign.id,
            character_id,
        })?;

    let modifier =
        manager::get_character_roll_stats(&***db, &campaign, &character, roll_type).await?;

    Ok(Json(RollStatsBody { modifier }))
}
