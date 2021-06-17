use actix_web::web::{Data, Json, Path};
use actix_web::{get, post};
use mongodb::Database;
use serde::{Deserialize, Serialize};

use crate::error::Error;
use crate::{db, Campaign, CampaignId, Character, CharacterId, CharacterOwner};

#[derive(Clone, Debug, Deserialize)]
struct CreateCampaignBody {
    name: String,
}

#[derive(Clone, Debug, Serialize)]
struct CampaignBody {
    id: CampaignId,
    name: String,
    characters: Vec<CharacterBody>,
}

#[post("/campaigns")]
#[tracing::instrument(skip(db))]
async fn create_campaign(
    db: Data<Database>,
    body: Json<CreateCampaignBody>,
) -> Result<Json<CampaignBody>, Error> {
    let body = body.into_inner();
    let campaign = Campaign {
        id: uuid::Uuid::new_v4().to_string(),
        name: body.name,
    };

    db::insert_campaign(&db, &campaign).await?;

    let body = CampaignBody {
        id: campaign.id,
        name: campaign.name,
        characters: vec![],
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
        });
    }

    Ok(Json(body))
}

#[get("/campaigns/{campaign_id}")]
#[tracing::instrument(skip(db))]
async fn get_campaign_by_id(
    db: Data<Database>,
    params: Path<String>,
) -> Result<Json<Option<CampaignBody>>, Error> {
    let campaign_id = params.into_inner();

    let campaign = db::fetch_campaign_by_id(&db, campaign_id).await?;

    let body = CampaignBody {
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
async fn create_character(
    db: Data<Database>,
    params: Path<String>,
    body: Json<CreateCharacterBody>,
) -> Result<Json<CharacterBody>, Error> {
    let campaign_id = params.into_inner();
    let body = body.into_inner();
    let character = Character {
        id: uuid::Uuid::new_v4().to_string(),
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
async fn get_characters(
    db: Data<Database>,
    params: Path<String>,
) -> Result<Json<Vec<CharacterBody>>, Error> {
    let campaign_id = params.into_inner();

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
