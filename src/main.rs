use std::io::{Error, ErrorKind};

use actix_web::web::{Data, Json, Path};
use actix_web::{get, post, App, HttpServer};
use mongodb::{bson, Client, Database};
use serde::de::Error as DeError;
use serde::{Deserialize, Serialize};
use tracing::info;
use tracing_actix_web::TracingLogger;
use tracing_subscriber::fmt::format::FmtSpan;

mod db;

type CampaignId = String;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Campaign {
    #[serde(rename = "_id")]
    id: CampaignId,
    name: String,
}

#[derive(Clone, Debug, Deserialize)]
struct CreateCampaignBody {
    name: String,
}

#[derive(Clone, Debug, Serialize)]
struct CampaignBody {
    id: CampaignId,
    name: String,
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
    };

    Ok(Json(body))
}

#[get("/campaigns")]
#[tracing::instrument(skip(db))]
async fn get_campaigns(db: Data<Database>) -> Result<Json<Vec<CampaignBody>>, Error> {
    let campaigns = db::fetch_campaigns(&db).await?;

    let body = campaigns
        .into_iter()
        .map(|campaign| CampaignBody {
            id: campaign.id,
            name: campaign.name,
        })
        .collect();

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

    let body = campaign.map(|campaign| CampaignBody {
        id: campaign.id,
        name: campaign.name,
    });

    Ok(Json(body))
}

type UserId = String;
type CharacterId = String;

// A character must have an owning User, Campaign, or both
#[derive(Clone, Debug)]
enum CharacterOwner {
    Campaign(CampaignId),
    User(UserId),
    UserInCampaign(UserId, CampaignId),
}

impl Serialize for CharacterOwner {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        #[derive(Serialize)]
        struct Dummy<'a> {
            campaign_id: Option<&'a CampaignId>,
            user_id: Option<&'a UserId>,
        }

        let dummy = match self {
            CharacterOwner::Campaign(campaign_id) => Dummy {
                campaign_id: Some(campaign_id),
                user_id: None,
            },
            CharacterOwner::User(user_id) => Dummy {
                campaign_id: None,
                user_id: Some(user_id),
            },
            CharacterOwner::UserInCampaign(user_id, campaign_id) => Dummy {
                campaign_id: Some(campaign_id),
                user_id: Some(user_id),
            },
        };

        dummy.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for CharacterOwner {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Dummy {
            campaign_id: Option<CampaignId>,
            user_id: Option<UserId>,
        }

        let dummy = Dummy::deserialize(deserializer)?;

        match (dummy.campaign_id, dummy.user_id) {
            (Some(campaign_id), None) => Ok(CharacterOwner::Campaign(campaign_id)),
            (None, Some(user_id)) => Ok(CharacterOwner::User(user_id)),
            (Some(campaign_id), Some(user_id)) => {
                Ok(CharacterOwner::UserInCampaign(user_id, campaign_id))
            }
            (None, None) => Err(D::Error::custom("character must have a user or campaign")),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Character {
    #[serde(rename = "_id")]
    id: CharacterId,
    owner: CharacterOwner,
    name: String,
    // attributes: String,
    // items: Vec<Item>,
    // position: Option<(f32, f32)>,
    // health: u32,
    // effects: Vec<Effect>,
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

#[actix_web::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_span_events(FmtSpan::NEW)
        .pretty()
        .init();

    let uri = "mongodb://localhost:27017";
    info!("connecting to db: {}", uri);
    let db = Client::with_uri_str(uri)
        .await
        .map_err(|err| Error::new(ErrorKind::Other, err))?
        .database("dnd");

    // ping the database to ensure connection is established
    db.run_command(bson::doc! { "ping": 1 }, None)
        .await
        .map_err(|err| Error::new(ErrorKind::Other, err))?;

    HttpServer::new(move || {
        App::new()
            .data(db.clone())
            .wrap(TracingLogger::default())
            .service(create_campaign)
            .service(get_campaigns)
            .service(get_campaign_by_id)
            .service(create_character)
            .service(get_characters)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
