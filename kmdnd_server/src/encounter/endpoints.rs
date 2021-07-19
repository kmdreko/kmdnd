use actix_web::web::{Data, Json, Path};
use actix_web::{get, post};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::campaign::{self, CampaignId};
use crate::character::CharacterId;
use crate::database::Database;
use crate::error::Error;
use crate::utils::SuccessBody;

use super::{manager, Encounter, EncounterId, EncounterState};

#[derive(Clone, Debug, Deserialize)]
pub struct CreateEncounterBody {
    pub character_ids: Vec<CharacterId>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EncounterBody {
    pub id: EncounterId,
    pub campaign_id: CampaignId,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
    pub character_ids: Vec<CharacterId>,
    pub state: EncounterState,
}

impl EncounterBody {
    pub fn render(encounter: Encounter) -> EncounterBody {
        EncounterBody {
            id: encounter.id,
            campaign_id: encounter.campaign_id,
            created_at: encounter.created_at,
            modified_at: encounter.modified_at,
            character_ids: encounter.character_ids,
            state: encounter.state,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct BeginEncounterResultBody {
    turn_order: Vec<CharacterId>,
}

#[post("/campaigns/{campaign_id}/encounters")]
#[tracing::instrument(skip(db))]
async fn create_encounter_in_campaign(
    db: Data<Box<dyn Database>>,
    params: Path<CampaignId>,
    body: Json<CreateEncounterBody>,
) -> Result<Json<EncounterBody>, Error> {
    let campaign_id = params.into_inner();
    let campaign = campaign::manager::get_campaign_by_id(&***db, campaign_id)
        .await?
        .ok_or(Error::CampaignNotFound { campaign_id })?;
    let body = body.into_inner();

    let encounter = manager::create_encounter(&***db, &campaign, body.character_ids).await?;

    Ok(Json(EncounterBody::render(encounter)))
}

#[get("/campaigns/{campaign_id}/encounters")]
#[tracing::instrument(skip(db))]
async fn get_encounters_in_campaign(
    db: Data<Box<dyn Database>>,
    params: Path<CampaignId>,
) -> Result<Json<Vec<EncounterBody>>, Error> {
    let campaign_id = params.into_inner();
    let campaign = campaign::manager::get_campaign_by_id(&***db, campaign_id)
        .await?
        .ok_or(Error::CampaignNotFound { campaign_id })?;

    let encounters = manager::get_encounters(&***db, &campaign).await?;

    let body = encounters.into_iter().map(EncounterBody::render).collect();

    Ok(Json(body))
}

#[get("/campaigns/{campaign_id}/encounters/CURRENT")]
#[tracing::instrument(skip(db))]
async fn get_current_encounter_in_campaign(
    db: Data<Box<dyn Database>>,
    params: Path<CampaignId>,
) -> Result<Json<EncounterBody>, Error> {
    let campaign_id = params.into_inner();
    let campaign = campaign::manager::get_campaign_by_id(&***db, campaign_id)
        .await?
        .ok_or(Error::CampaignNotFound { campaign_id })?;
    let encounter = manager::get_current_encounter(&***db, &campaign)
        .await?
        .ok_or(Error::CurrentEncounterNotFound {
            campaign_id: campaign.id,
        })?;

    Ok(Json(EncounterBody::render(encounter)))
}

#[post("/campaigns/{campaign_id}/encounters/CURRENT/begin")]
#[tracing::instrument(skip(db))]
async fn begin_current_encounter_in_campaign(
    db: Data<Box<dyn Database>>,
    params: Path<CampaignId>,
) -> Result<Json<BeginEncounterResultBody>, Error> {
    let campaign_id = params.into_inner();
    let campaign = campaign::manager::get_campaign_by_id(&***db, campaign_id)
        .await?
        .ok_or(Error::CampaignNotFound { campaign_id })?;
    let encounter = manager::get_current_encounter(&***db, &campaign)
        .await?
        .ok_or(Error::CurrentEncounterNotFound {
            campaign_id: campaign.id,
        })?;

    let turn_order = manager::begin_encounter(&***db, &campaign, encounter).await?;

    let body = BeginEncounterResultBody { turn_order };

    Ok(Json(body))
}

#[post("/campaigns/{campaign_id}/encounters/CURRENT/finish")]
#[tracing::instrument(skip(db))]
async fn finish_current_encounter_in_campaign(
    db: Data<Box<dyn Database>>,
    params: Path<CampaignId>,
) -> Result<Json<SuccessBody>, Error> {
    let campaign_id = params.into_inner();
    let campaign = campaign::manager::get_campaign_by_id(&***db, campaign_id)
        .await?
        .ok_or(Error::CampaignNotFound { campaign_id })?;
    let encounter = manager::get_current_encounter(&***db, &campaign)
        .await?
        .ok_or(Error::CurrentEncounterNotFound {
            campaign_id: campaign.id,
        })?;

    manager::finish_encounter(&***db, &campaign, encounter).await?;

    Ok(Json(SuccessBody {}))
}
