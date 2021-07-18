use kmdnd_server::Error;
use tracing::Level;
use tracing_subscriber::fmt::format::FmtSpan;

fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .with_span_events(FmtSpan::NEW)
        .compact()
        .init();

    kmdnd_server::run(true)
}
