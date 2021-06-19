use actix_web::web::{Data, Json, Path};
use actix_web::{get, post};
use mongodb::Database;
use serde::{Deserialize, Serialize};

use super::{db, Campaign, CampaignId};
use crate::character::{self, CharacterBody};
use crate::encounter::{self, EncounterBody};
use crate::error::Error;

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
            characters: character::db::fetch_characters_by_campaign(&db, campaign.id)
                .await?
                .into_iter()
                .map(|character| CharacterBody {
                    id: character.id,
                    name: character.name,
                    owner: character.owner,
                })
                .collect(),
            current_encounter: encounter::db::fetch_current_encounter_by_campaign(&db, campaign.id)
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
        characters: character::db::fetch_characters_by_campaign(&db, campaign.id)
            .await?
            .into_iter()
            .map(|character| CharacterBody {
                id: character.id,
                name: character.name,
                owner: character.owner,
            })
            .collect(),
        current_encounter: encounter::db::fetch_current_encounter_by_campaign(&db, campaign.id)
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
