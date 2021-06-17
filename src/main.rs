use std::io::{Error, ErrorKind};

use actix_web::{App, HttpServer};
use mongodb::{bson, Client};
use serde::de::Error as DeError;
use serde::{Deserialize, Serialize};
use tracing::info;
use tracing_actix_web::TracingLogger;
use tracing_subscriber::fmt::format::FmtSpan;

mod db;
mod error;
mod handlers;

type CampaignId = String;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Campaign {
    #[serde(rename = "_id")]
    id: CampaignId,
    name: String,
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

#[actix_web::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_span_events(FmtSpan::NEW)
        .compact()
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
            .service(handlers::create_campaign)
            .service(handlers::get_campaigns)
            .service(handlers::get_campaign_by_id)
            .service(handlers::create_character)
            .service(handlers::get_characters)
            .service(handlers::get_character_by_campaign)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
