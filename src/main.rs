use sqlx::PgPool;
use tracing::subscriber::set_global_default;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use std::net::TcpListener;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};

use zero2prod::{
    configuration::{get_configuration, Settings},
    startup::run,
};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let formatting_layer = BunyanFormattingLayer::new("zero2prod".into(), std::io::stdout);

    // We've created a subscriber's logging pipeline
    let subscriber = Registry::default()
        .with(env_filter) // ?
        .with(JsonStorageLayer) // For some reason this takes incoming spans and logs and stores them in JSON
        .with(formatting_layer); // this will format the previous JSON into bunyan(WTF is bunayn?

    set_global_default(subscriber).expect("Failed to set subscriber");

    // Configuration is retrieved from `configuration.yaml` file
    let configuration = get_configuration().expect("Failed to read configuration");
    let Settings {
        application_port,
        database,
    } = configuration;

    // `actix-web` creates a worker per CPU core. Workers utilize connections
    // which are taken from the connection pool instead of creating a connection per
    // request as an optimization technique
    let connection_pool = PgPool::connect(&database.connection_string())
        .await
        .expect("Failed to connect to Postgres");

    let address = format!("127.0.0.1:{}", application_port);
    let listener = TcpListener::bind(address).expect("Failed to bind address");

    run(listener, connection_pool)?.await
}
