use actix_web::web::{Data, Json, Path};
use actix_web::{get, post};
use chrono::{DateTime, Utc};
use mongodb::Database;
use serde::{Deserialize, Serialize};

use crate::campaign::{self, CampaignId};
use crate::character::{self, CharacterId};
use crate::error::Error;

use super::{db, Encounter, EncounterId, EncounterState};

#[derive(Clone, Debug, Serialize)]
struct SuccessBody {}

#[derive(Clone, Debug, Deserialize)]
pub struct CreateEncounterBody {
    pub character_ids: Vec<CharacterId>,
}

#[derive(Clone, Debug, Serialize)]
pub struct EncounterBody {
    pub id: EncounterId,
    pub campaign_id: CampaignId,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
    pub character_ids: Vec<CharacterId>,
    pub state: EncounterState,
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

    campaign::db::fetch_campaign_by_id(&db, campaign_id)
        .await?
        .ok_or(Error::CampaignDoesNotExist(campaign_id))?;

    let current_encounter = db::fetch_current_encounter_by_campaign(&db, campaign_id).await?;
    if let Some(current_encounter) = current_encounter {
        return Err(Error::CurrentEncounterAlreadyExists(current_encounter.id));
    }

    let characters = character::db::fetch_characters_by_campaign(&db, campaign_id).await?;
    for character_id in &body.character_ids {
        if !characters.iter().any(|c| c.id == *character_id) {
            return Err(Error::CharacterNotInCampaign(*character_id));
        }
    }

    let now = Utc::now();
    let encounter = Encounter {
        id: EncounterId::new(),
        campaign_id,
        created_at: now,
        modified_at: now,
        character_ids: body.character_ids,
        state: EncounterState::Initiative,
    };

    db::insert_encounter(&db, &encounter).await?;

    let body = EncounterBody {
        id: encounter.id,
        campaign_id: encounter.campaign_id,
        created_at: encounter.created_at,
        modified_at: encounter.modified_at,
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

    campaign::db::fetch_campaign_by_id(&db, campaign_id)
        .await?
        .ok_or(Error::CampaignDoesNotExist(campaign_id))?;

    let encounters = db::fetch_encounters_by_campaign(&db, campaign_id).await?;

    let body = encounters
        .into_iter()
        .map(|encounter| EncounterBody {
            id: encounter.id,
            campaign_id: encounter.campaign_id,
            created_at: encounter.created_at,
            modified_at: encounter.modified_at,
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

    campaign::db::fetch_campaign_by_id(&db, campaign_id)
        .await?
        .ok_or(Error::CampaignDoesNotExist(campaign_id))?;

    let encounter = db::fetch_current_encounter_by_campaign(&db, campaign_id)
        .await?
        .ok_or(Error::CurrentEncounterDoesNotExist)?;

    let body = EncounterBody {
        id: encounter.id,
        campaign_id: encounter.campaign_id,
        created_at: encounter.created_at,
        modified_at: encounter.modified_at,
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

    campaign::db::fetch_campaign_by_id(&db, campaign_id)
        .await?
        .ok_or(Error::CampaignDoesNotExist(campaign_id))?;

    let encounter = db::fetch_current_encounter_by_campaign(&db, campaign_id)
        .await?
        .ok_or(Error::CurrentEncounterDoesNotExist)?;

    db::update_encounter_state(&db, &encounter, EncounterState::Finished).await?;

    Ok(Json(SuccessBody {}))
}
