use actix_web::web::{Data, Json, Path};
use actix_web::{get, post};
use chrono::{DateTime, Utc};
use mongodb::Database;
use serde::{Deserialize, Serialize};

use crate::campaign::{self, CampaignId};
use crate::character::{self, Character, CharacterId, CharacterOwner};
use crate::error::Error;

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
}

impl CharacterBody {
    pub fn render(character: Character) -> CharacterBody {
        CharacterBody {
            id: character.id,
            owner: character.owner,
            name: character.name,
            created_at: character.created_at,
            modified_at: character.modified_at,
        }
    }
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
    };

    character::db::insert_character(&db, &character).await?;

    Ok(Json(CharacterBody::render(character)))
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

    let characters = character::db::fetch_characters_by_campaign(&db, campaign_id).await?;

    let body = characters
        .into_iter()
        .map(|character| CharacterBody::render(character))
        .collect();

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

    let character =
        character::db::fetch_character_by_campaign_and_id(&db, campaign_id, character_id)
            .await?
            .ok_or(Error::CharacterDoesNotExistInCampaign {
                campaign_id,
                character_id,
            })?;

    Ok(Json(CharacterBody::render(character)))
}
