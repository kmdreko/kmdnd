use std::io::{Error, ErrorKind};

use actix_web::{App, HttpServer};
use mongodb::{bson, Client};
use tracing::info;
use tracing_actix_web::TracingLogger;
use tracing_subscriber::fmt::format::FmtSpan;

mod campaign;
mod character;
mod encounter;
mod error;
mod operation;
mod typedid;
mod user;

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
            .service(campaign::endpoints::create_campaign)
            .service(campaign::endpoints::get_campaigns)
            .service(campaign::endpoints::get_campaign_by_id)
            .service(character::endpoints::create_character_in_campaign)
            .service(character::endpoints::get_characters_in_campaign)
            .service(character::endpoints::get_character_in_campaign_by_id)
            .service(encounter::endpoints::create_encounter_in_campaign)
            .service(encounter::endpoints::get_encounters_in_campaign)
            .service(encounter::endpoints::get_current_encounter_in_campaign)
            .service(encounter::endpoints::finish_current_encounter_in_campaign)
            .service(operation::endpoints::get_operations_in_current_encounter_in_campaign)
            .service(operation::endpoints::move_in_current_encounter_in_campaign)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
