use std::io::{Error, ErrorKind};

use actix_web::web::{Data, Json};
use actix_web::{get, post, App, HttpServer};
use futures::TryStreamExt;
use mongodb::{bson, Client, Database};
use serde::{Deserialize, Serialize};
use tracing::info;
use tracing_actix_web::TracingLogger;
use tracing_subscriber::fmt::format::FmtSpan;

type CampaignId = String;

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Campaign {
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
#[tracing::instrument]
async fn create_campaign(
    db: Data<Database>,
    body: Json<CreateCampaignBody>,
) -> Result<Json<CampaignBody>, Error> {
    let body = body.into_inner();
    let campaign = Campaign {
        id: uuid::Uuid::new_v4().to_string(),
        name: body.name,
    };

    let doc = bson::to_document(&campaign).map_err(|err| Error::new(ErrorKind::Other, err))?;
    db.collection("campaigns")
        .insert_one(doc, None)
        .await
        .map_err(|err| Error::new(ErrorKind::Other, err))?;

    let body = CampaignBody {
        id: campaign.id,
        name: campaign.name,
    };

    Ok(Json(body))
}

#[get("/campaigns")]
#[tracing::instrument]
async fn get_campaigns(db: Data<Database>) -> Result<Json<Vec<CampaignBody>>, Error> {
    let campaigns: Vec<Campaign> = db
        .collection("campaigns")
        .find(bson::doc! {}, None)
        .await
        .map_err(|err| Error::new(ErrorKind::Other, err))?
        .try_collect()
        .await
        .map_err(|err| Error::new(ErrorKind::Other, err))?;

    let body = campaigns
        .into_iter()
        .map(|c| CampaignBody {
            id: c.id,
            name: c.name,
        })
        .collect();

    Ok(Json(body))
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
            .service(create_campaign)
            .service(get_campaigns)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
