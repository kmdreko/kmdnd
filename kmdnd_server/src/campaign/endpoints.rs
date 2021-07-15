use actix_web::web::{Data, Json, Path};
use actix_web::{get, post};
use chrono::{DateTime, Utc};
use futures::{stream, StreamExt, TryStreamExt};
use serde::{Deserialize, Serialize};

use crate::character::{self, CharacterBody};
use crate::database::MongoDatabase;
use crate::encounter::{self, EncounterBody};
use crate::error::Error;

use super::{db, Campaign, CampaignId};

#[derive(Clone, Debug, Deserialize)]
struct CreateCampaignBody {
    pub name: String,
}

#[derive(Clone, Debug, Serialize)]
struct CampaignBody {
    pub id: CampaignId,
    pub name: String,
    pub characters: Vec<CharacterBody>,
    pub current_encounter: Option<EncounterBody>,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
}

impl CampaignBody {
    pub async fn render(db: &MongoDatabase, campaign: Campaign) -> Result<CampaignBody, Error> {
        let characters =
            character::db::fetch_characters_by_campaign(db.characters(), campaign.id).await?;
        Ok(CampaignBody {
            id: campaign.id,
            name: campaign.name,
            created_at: campaign.created_at,
            modified_at: campaign.modified_at,
            characters: stream::iter(characters)
                .then(|character| CharacterBody::render(db, character))
                .try_collect()
                .await?,
            current_encounter: encounter::db::fetch_current_encounter_by_campaign(
                db.encounters(),
                campaign.id,
            )
            .await?
            .map(|encounter| EncounterBody::render(encounter)),
        })
    }
}

#[post("/campaigns")]
#[tracing::instrument(skip(db))]
async fn create_campaign(
    db: Data<MongoDatabase>,
    body: Json<CreateCampaignBody>,
) -> Result<Json<CampaignBody>, Error> {
    let body = body.into_inner();

    let now = Utc::now();
    let campaign = Campaign {
        id: CampaignId::new(),
        name: body.name,
        created_at: now,
        modified_at: now,
    };

    db::insert_campaign(db.campaigns(), &campaign).await?;

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
async fn get_campaigns(db: Data<MongoDatabase>) -> Result<Json<Vec<CampaignBody>>, Error> {
    let campaigns = db::fetch_campaigns(db.campaigns()).await?;

    let body = stream::iter(campaigns)
        .then(|campaign| CampaignBody::render(&db, campaign))
        .try_collect()
        .await?;

    Ok(Json(body))
}

#[get("/campaigns/{campaign_id}")]
#[tracing::instrument(skip(db))]
async fn get_campaign_by_id(
    db: Data<MongoDatabase>,
    params: Path<CampaignId>,
) -> Result<Json<CampaignBody>, Error> {
    let campaign_id = params.into_inner();

    let campaign = db::fetch_campaign_by_id(db.campaigns(), campaign_id)
        .await?
        .ok_or(Error::CampaignDoesNotExist { campaign_id })?;

    Ok(Json(CampaignBody::render(&db, campaign).await?))
}
