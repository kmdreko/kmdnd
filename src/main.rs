use std::io::{Error, ErrorKind};

use actix_web::{App, HttpServer};
use mongodb::{bson, Client};
use tracing::info;
use tracing_actix_web::TracingLogger;
use tracing_subscriber::fmt::format::FmtSpan;

mod campaign;
mod character;
mod db;
mod encounter;
mod error;
mod handlers;
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
            .service(handlers::create_campaign)
            .service(handlers::get_campaigns)
            .service(handlers::get_campaign_by_id)
            .service(handlers::create_character_in_campaign)
            .service(handlers::get_characters_in_campaign)
            .service(handlers::get_character_in_campaign_by_id)
            .service(handlers::create_encounter_in_campaign)
            .service(handlers::get_encounters_in_campaign)
            .service(handlers::get_current_encounter_in_campaign)
            .service(handlers::finish_current_encounter_in_campaign)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
