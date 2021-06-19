use actix_web::web::{Data, Json, Path};
use actix_web::{get, post};
use chrono::{DateTime, Utc};
use mongodb::Database;
use serde::{Deserialize, Serialize};

use crate::campaign::{Campaign, CampaignId};
use crate::character::{Character, CharacterId, CharacterOwner};
use crate::db;
use crate::encounter::{Encounter, EncounterId, EncounterState};
use crate::error::Error;

#[derive(Clone, Debug, Serialize)]
struct SuccessBody {}

#[derive(Clone, Debug, Deserialize)]
struct CreateCampaignBody {
    name: String,
}

#[derive(Clone, Debug, Serialize)]
struct CampaignBody {
    id: CampaignId,
    name: String,
    characters: Vec<CharacterBody>,
    current_encounter: Option<EncounterBody>,
}

#[post("/campaigns")]
#[tracing::instrument(skip(db))]
async fn create_campaign(
    db: Data<Database>,
    body: Json<CreateCampaignBody>,
) -> Result<Json<CampaignBody>, Error> {
    let body = body.into_inner();
    let campaign = Campaign {
        id: CampaignId::new(),
        name: body.name,
    };

    db::insert_campaign(&db, &campaign).await?;

    let body = CampaignBody {
        id: campaign.id,
        name: campaign.name,
        characters: vec![],
        current_encounter: None,
    };

    Ok(Json(body))
}

#[get("/campaigns")]
#[tracing::instrument(skip(db))]
async fn get_campaigns(db: Data<Database>) -> Result<Json<Vec<CampaignBody>>, Error> {
    let campaigns = db::fetch_campaigns(&db).await?;

    let mut body = vec![];
    for campaign in campaigns {
        body.push(CampaignBody {
            id: campaign.id.clone(),
            name: campaign.name,
            characters: db::fetch_characters_by_campaign(&db, campaign.id)
                .await?
                .into_iter()
                .map(|character| CharacterBody {
                    id: character.id,
                    name: character.name,
                    owner: character.owner,
                })
                .collect(),
            current_encounter: db::fetch_current_encounter_by_campaign(&db, campaign.id)
                .await?
                .map(|encounter| EncounterBody {
                    id: encounter.id,
                    campaign_id: encounter.campaign_id,
                    created_at: encounter.created_at,
                    character_ids: encounter.character_ids,
                    state: encounter.state,
                }),
        });
    }

    Ok(Json(body))
}

#[get("/campaigns/{campaign_id}")]
#[tracing::instrument(skip(db))]
async fn get_campaign_by_id(
    db: Data<Database>,
    params: Path<CampaignId>,
) -> Result<Json<CampaignBody>, Error> {
    let campaign_id = params.into_inner();

    let campaign = db::fetch_campaign_by_id(&db, campaign_id)
        .await?
        .ok_or(Error::CampaignDoesNotExist(campaign_id))?;

    let body = CampaignBody {
        id: campaign.id,
        name: campaign.name,
        characters: db::fetch_characters_by_campaign(&db, campaign.id)
            .await?
            .into_iter()
            .map(|character| CharacterBody {
                id: character.id,
                name: character.name,
                owner: character.owner,
            })
            .collect(),
        current_encounter: db::fetch_current_encounter_by_campaign(&db, campaign.id)
            .await?
            .map(|encounter| EncounterBody {
                id: encounter.id,
                campaign_id: encounter.campaign_id,
                created_at: encounter.created_at,
                character_ids: encounter.character_ids,
                state: encounter.state,
            }),
    };

    Ok(Json(body))
}

#[derive(Clone, Debug, Deserialize)]
struct CreateCharacterBody {
    name: String,
}

#[derive(Clone, Debug, Serialize)]
struct CharacterBody {
    id: CharacterId,
    owner: CharacterOwner,
    name: String,
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

    db::fetch_campaign_by_id(&db, campaign_id)
        .await?
        .ok_or(Error::CampaignDoesNotExist(campaign_id))?;

    let character = Character {
        id: CharacterId::new(),
        owner: CharacterOwner::Campaign(campaign_id),
        name: body.name,
    };

    db::insert_character(&db, &character).await?;

    let body = CharacterBody {
        id: character.id,
        owner: character.owner,
        name: character.name,
    };

    Ok(Json(body))
}

#[get("/campaigns/{campaign_id}/characters")]
#[tracing::instrument(skip(db))]
async fn get_characters_in_campaign(
    db: Data<Database>,
    params: Path<CampaignId>,
) -> Result<Json<Vec<CharacterBody>>, Error> {
    let campaign_id = params.into_inner();

    db::fetch_campaign_by_id(&db, campaign_id)
        .await?
        .ok_or(Error::CampaignDoesNotExist(campaign_id))?;

    let characters = db::fetch_characters_by_campaign(&db, campaign_id).await?;

    let body = characters
        .into_iter()
        .map(|character| CharacterBody {
            id: character.id,
            owner: character.owner,
            name: character.name,
        })
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

    db::fetch_campaign_by_id(&db, campaign_id)
        .await?
        .ok_or(Error::CampaignDoesNotExist(campaign_id))?;

    let character = db::fetch_character_by_campaign_and_id(&db, campaign_id, character_id)
        .await?
        .ok_or(Error::CharacterDoesNotExist(character_id))?;

    let body = CharacterBody {
        id: character.id,
        owner: character.owner,
        name: character.name,
    };

    Ok(Json(body))
}

#[derive(Clone, Debug, Deserialize)]
struct CreateEncounterBody {
    character_ids: Vec<CharacterId>,
}

#[derive(Clone, Debug, Serialize)]
struct EncounterBody {
    id: EncounterId,
    campaign_id: CampaignId,
    created_at: DateTime<Utc>,
    character_ids: Vec<CharacterId>,
    state: EncounterState,
}

#[post("/campaigns/{campaign_id}/encounters")]
#[tracing::instrument(skip(db))]
async fn create_encounter_in_campaign(
    db: Data<Database>,
    params: Path<CampaignId>,
    body: Json<CreateEncounterBody>,
) -> Result<Json<EncounterBody>, Error> {
    let campaign_id = params.into_inner();
    let body = body.into_inner();

    db::fetch_campaign_by_id(&db, campaign_id)
        .await?
        .ok_or(Error::CampaignDoesNotExist(campaign_id))?;

    let current_encounter = db::fetch_current_encounter_by_campaign(&db, campaign_id).await?;
    if let Some(current_encounter) = current_encounter {
        return Err(Error::CurrentEncounterAlreadyExists(current_encounter.id));
    }

    let encounter = Encounter {
        id: EncounterId::new(),
        campaign_id,
        created_at: Utc::now(),
        character_ids: body.character_ids,
        state: EncounterState::Initiative,
    };

    db::insert_encounter(&db, &encounter).await?;

    let body = EncounterBody {
        id: encounter.id,
        campaign_id: encounter.campaign_id,
        created_at: encounter.created_at,
        character_ids: encounter.character_ids,
        state: encounter.state,
    };

    Ok(Json(body))
}

#[get("/campaigns/{campaign_id}/encounters")]
#[tracing::instrument(skip(db))]
async fn get_encounters_in_campaign(
    db: Data<Database>,
    params: Path<CampaignId>,
) -> Result<Json<Vec<EncounterBody>>, Error> {
    let campaign_id = params.into_inner();

    db::fetch_campaign_by_id(&db, campaign_id)
        .await?
        .ok_or(Error::CampaignDoesNotExist(campaign_id))?;

    let encounters = db::fetch_encounters_by_campaign(&db, campaign_id).await?;

    let body = encounters
        .into_iter()
        .map(|encounter| EncounterBody {
            id: encounter.id,
            campaign_id: encounter.campaign_id,
            created_at: encounter.created_at,
            character_ids: encounter.character_ids,
            state: encounter.state,
        })
        .collect();

    Ok(Json(body))
}

#[get("/campaigns/{campaign_id}/encounters/current")]
#[tracing::instrument(skip(db))]
async fn get_current_encounter_in_campaign(
    db: Data<Database>,
    params: Path<CampaignId>,
) -> Result<Json<EncounterBody>, Error> {
    let campaign_id = params.into_inner();

    db::fetch_campaign_by_id(&db, campaign_id)
        .await?
        .ok_or(Error::CampaignDoesNotExist(campaign_id))?;

    let encounter = db::fetch_current_encounter_by_campaign(&db, campaign_id)
        .await?
        .ok_or(Error::CurrentEncounterDoesNotExist)?;

    let body = EncounterBody {
        id: encounter.id,
        campaign_id: encounter.campaign_id,
        created_at: encounter.created_at,
        character_ids: encounter.character_ids,
        state: encounter.state,
    };

    Ok(Json(body))
}

#[post("/campaigns/{campaign_id}/encounters/current/finish")]
#[tracing::instrument(skip(db))]
async fn finish_current_encounter_in_campaign(
    db: Data<Database>,
    params: Path<CampaignId>,
) -> Result<Json<SuccessBody>, Error> {
    let campaign_id = params.into_inner();

    db::fetch_campaign_by_id(&db, campaign_id)
        .await?
        .ok_or(Error::CampaignDoesNotExist(campaign_id))?;

    let encounter = db::fetch_current_encounter_by_campaign(&db, campaign_id)
        .await?
        .ok_or(Error::CurrentEncounterDoesNotExist)?;

    db::update_encounter_state(&db, encounter.id, EncounterState::Finished).await?;

    Ok(Json(SuccessBody {}))
}
