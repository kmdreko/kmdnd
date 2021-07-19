use actix_web::web::{Data, Json, Path};
use actix_web::{get, post};
use chrono::{DateTime, Utc};
use futures::{stream, StreamExt, TryStreamExt};
use serde::{Deserialize, Serialize};

use crate::character::CharacterBody;
use crate::database::Database;
use crate::encounter::EncounterBody;
use crate::error::Error;

use super::{manager, Campaign, CampaignId};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CreateCampaignBody {
    pub name: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CampaignBody {
    pub id: CampaignId,
    pub name: String,
    characters: Vec<CharacterBody>,
    current_encounter: Option<EncounterBody>,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
}

impl CampaignBody {
    pub async fn render(db: &dyn Database, campaign: Campaign) -> Result<CampaignBody, Error> {
        let characters = db
            .characters()
            .fetch_characters_by_campaign(campaign.id)
            .await?;
        let characters = stream::iter(characters)
            .then(|character| CharacterBody::render(db, character))
            .try_collect()
            .await?;

        Ok(CampaignBody {
            id: campaign.id,
            name: campaign.name,
            created_at: campaign.created_at,
            modified_at: campaign.modified_at,
            characters,
            current_encounter: db
                .encounters()
                .fetch_current_encounter_by_campaign(campaign.id)
                .await?
                .map(EncounterBody::render),
        })
    }
}

#[post("/campaigns")]
#[tracing::instrument(skip(db))]
async fn create_campaign(
    db: Data<Box<dyn Database>>,
    body: Json<CreateCampaignBody>,
) -> Result<Json<CampaignBody>, Error> {
    let body = body.into_inner();

    let campaign = manager::create_campaign(&***db, body.name).await?;

    let body = CampaignBody {
        id: campaign.id,
        name: campaign.name,
        created_at: campaign.created_at,
        modified_at: campaign.modified_at,
        characters: vec![],
        current_encounter: None,
    };

    Ok(Json(body))
}

#[get("/campaigns")]
#[tracing::instrument(skip(db))]
async fn get_campaigns(db: Data<Box<dyn Database>>) -> Result<Json<Vec<CampaignBody>>, Error> {
    let campaigns = manager::get_campaigns(&***db).await?;

    let body = stream::iter(campaigns)
        .then(|campaign| CampaignBody::render(&***db, campaign))
        .try_collect()
        .await?;

    Ok(Json(body))
}

#[get("/campaigns/{campaign_id}")]
#[tracing::instrument(skip(db))]
async fn get_campaign_by_id(
    db: Data<Box<dyn Database>>,
    params: Path<CampaignId>,
) -> Result<Json<CampaignBody>, Error> {
    let campaign_id = params.into_inner();
    let campaign = manager::get_campaign_by_id(&***db, campaign_id)
        .await?
        .ok_or(Error::CampaignNotFound { campaign_id })?;

    Ok(Json(CampaignBody::render(&***db, campaign).await?))
}
