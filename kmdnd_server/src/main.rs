use actix_web::web::{self, FormConfig, JsonConfig, PathConfig, QueryConfig};
use actix_web::{App, HttpServer, ResponseError};
use mongodb::Client;
use tracing::{info, Level};
use tracing_actix_web::TracingLogger;
use tracing_subscriber::fmt::format::FmtSpan;

mod campaign;
mod character;
mod database;
mod encounter;
mod error;
mod item;
mod operation;
mod seed;
mod typedid;
mod user;
mod utils;
mod violations;

use error::Error;

use crate::database::{Database, MongoDatabase};

#[actix_web::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .with_span_events(FmtSpan::NEW)
        .compact()
        .init();

    let uri = "mongodb://localhost:27017";
    info!("connecting to db: {}", uri);
    let db = Client::with_uri_str(uri).await?.database("kmdnd");
    let db = MongoDatabase::initialize(db).await?;

    seed::seed(&db).await?;

    HttpServer::new(move || {
        App::new()
            .app_data(JsonConfig::default().error_handler(|err, _req| {
                // format json errors with custom format
                Error::InvalidJson(err).into()
            }))
            .app_data(PathConfig::default().error_handler(|err, _req| {
                // format path errors with custom format
                Error::InvalidPath(err).into()
            }))
            .app_data(FormConfig::default().error_handler(|err, _req| {
                // format form errors with custom format
                Error::InvalidForm(err).into()
            }))
            .app_data(QueryConfig::default().error_handler(|err, _req| {
                // format query errors with custom format
                Error::InvalidQuery(err).into()
            }))
            .data(Box::new(db.clone()) as Box<dyn Database>)
            .wrap(TracingLogger::default())
            .service(campaign::endpoints::create_campaign)
            .service(campaign::endpoints::get_campaigns)
            .service(campaign::endpoints::get_campaign_by_id)
            .service(character::endpoints::create_character_in_campaign)
            .service(character::endpoints::get_characters_in_campaign)
            .service(character::endpoints::get_character_in_campaign_by_id)
            .service(character::endpoints::get_character_roll_stats)
            .service(encounter::endpoints::create_encounter_in_campaign)
            .service(encounter::endpoints::get_encounters_in_campaign)
            .service(encounter::endpoints::get_current_encounter_in_campaign)
            .service(encounter::endpoints::finish_current_encounter_in_campaign)
            .service(operation::endpoints::get_operations_in_current_encounter_in_campaign)
            .service(operation::endpoints::get_operation_by_id_in_current_encounter_in_campaign)
            .service(operation::endpoints::approve_illegal_operation)
            .service(operation::endpoints::reject_illegal_operation)
            .service(operation::endpoints::submit_interaction_result_to_operation)
            .service(operation::endpoints::roll_in_current_encounter_in_campaign)
            .service(operation::endpoints::begin_current_encounter_in_campaign)
            .service(operation::endpoints::move_in_current_encounter_in_campaign)
            .service(operation::endpoints::take_action_in_current_encounter_in_campaign)
            .service(item::endpoints::get_items)
            .service(item::endpoints::get_item_by_id)
            .default_service(web::to(|| Error::PathDoesNotExist.error_response()))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await?;

    Ok(())
}
