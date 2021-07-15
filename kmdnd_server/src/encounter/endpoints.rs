use actix_web::web::{Data, Json, Path};
use actix_web::{get, post};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::campaign::{self, CampaignId};
use crate::character::{self, CharacterId};
use crate::database::MongoDatabase;
use crate::error::Error;
use crate::utils::SuccessBody;

use super::{db, Encounter, EncounterId, EncounterState};

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

#[post("/campaigns/{campaign_id}/encounters")]
#[tracing::instrument(skip(db))]
async fn create_encounter_in_campaign(
    db: Data<MongoDatabase>,
    params: Path<CampaignId>,
    body: Json<CreateEncounterBody>,
) -> Result<Json<EncounterBody>, Error> {
    let campaign_id = params.into_inner();
    let campaign = campaign::db::assert_campaign_exists(db.campaigns(), campaign_id).await?;

    let body = body.into_inner();

    let current_encounter =
        db::fetch_current_encounter_by_campaign(db.encounters(), campaign.id).await?;
    if let Some(current_encounter) = current_encounter {
        return Err(Error::CurrentEncounterAlreadyExists {
            campaign_id: campaign.id,
            encounter_id: current_encounter.id,
        });
    }

    let characters =
        character::db::fetch_characters_by_campaign(db.characters(), campaign.id).await?;
    for character_id in &body.character_ids {
        if !characters.iter().any(|c| c.id == *character_id) {
            return Err(Error::CharacterNotInCampaign {
                campaign_id: campaign.id,
                character_id: *character_id,
            });
        }
    }

    let now = Utc::now();
    let encounter = Encounter {
        id: EncounterId::new(),
        campaign_id: campaign.id,
        created_at: now,
        modified_at: now,
        character_ids: body.character_ids,
        state: EncounterState::Initiative,
    };

    db::insert_encounter(db.encounters(), &encounter).await?;

    Ok(Json(EncounterBody::render(encounter)))
}

#[get("/campaigns/{campaign_id}/encounters")]
#[tracing::instrument(skip(db))]
async fn get_encounters_in_campaign(
    db: Data<MongoDatabase>,
    params: Path<CampaignId>,
) -> Result<Json<Vec<EncounterBody>>, Error> {
    let campaign_id = params.into_inner();
    let campaign = campaign::db::assert_campaign_exists(db.campaigns(), campaign_id).await?;

    let encounters = db::fetch_encounters_by_campaign(db.encounters(), campaign.id).await?;
    let body = encounters
        .into_iter()
        .map(|encounter| EncounterBody::render(encounter))
        .collect();

    Ok(Json(body))
}

#[get("/campaigns/{campaign_id}/encounters/CURRENT")]
#[tracing::instrument(skip(db))]
async fn get_current_encounter_in_campaign(
    db: Data<MongoDatabase>,
    params: Path<CampaignId>,
) -> Result<Json<EncounterBody>, Error> {
    let campaign_id = params.into_inner();
    let campaign = campaign::db::assert_campaign_exists(db.campaigns(), campaign_id).await?;

    let encounter = db::fetch_current_encounter_by_campaign(db.encounters(), campaign.id)
        .await?
        .ok_or(Error::CurrentEncounterDoesNotExist { campaign_id })?;

    Ok(Json(EncounterBody::render(encounter)))
}

#[post("/campaigns/{campaign_id}/encounters/CURRENT/finish")]
#[tracing::instrument(skip(db))]
async fn finish_current_encounter_in_campaign(
    db: Data<MongoDatabase>,
    params: Path<CampaignId>,
) -> Result<Json<SuccessBody>, Error> {
    let campaign_id = params.into_inner();
    let campaign = campaign::db::assert_campaign_exists(db.campaigns(), campaign_id).await?;
    let encounter = db::assert_current_encounter_exists(db.encounters(), campaign.id).await?;

    db::update_encounter_state(db.encounters(), encounter, EncounterState::Finished).await?;

    Ok(Json(SuccessBody {}))
}
