use actix_web::{get, App, HttpServer, Responder};
use tracing_actix_web::TracingLogger;
use tracing_subscriber::fmt::format::FmtSpan;

#[get("/test")]
#[tracing::instrument]
async fn test() -> impl Responder {
    "Hello world!"
}

#[actix_web::main]
async fn main() -> Result<(), std::io::Error> {
    tracing_subscriber::fmt()
        .with_span_events(FmtSpan::ENTER)
        .compact()
        .init();

    HttpServer::new(|| App::new().wrap(TracingLogger::default()).service(test))
        .bind("127.0.0.1:8080")?
        .run()
        .await
}
