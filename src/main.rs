use std::io::{Error, ErrorKind};

use actix_web::web::{Data, Json};
use actix_web::{get, App, HttpServer};
use futures::TryStreamExt;
use mongodb::{bson, Client, Database};
use serde::{Deserialize, Serialize};
use tracing::info;
use tracing_actix_web::TracingLogger;
use tracing_subscriber::fmt::format::FmtSpan;

#[derive(Debug, Clone, Deserialize, Serialize)]
struct Test {}

#[get("/test")]
#[tracing::instrument]
async fn test(db: Data<Database>) -> Result<Json<Vec<Test>>, Error> {
    let items: Vec<Test> = db
        .collection("test")
        .find(bson::doc! {}, None)
        .await
        .map_err(|err| Error::new(ErrorKind::Other, err))?
        .try_collect()
        .await
        .map_err(|err| Error::new(ErrorKind::Other, err))?;

    Ok(Json(items))
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
            .service(test)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
