use sqlx::PgPool;
use std::net::TcpListener;

use zero2prod::{
    configuration::{get_configuration, Settings},
    email_client::EmailClient,
    startup::run,
    telemetry::{get_subscriber, init_subscriber},
};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let subscriber = get_subscriber("zero2prod".to_string(), "info".to_string(), std::io::stdout);
    init_subscriber(subscriber);

    // Configuration is retrieved from `configuration.yaml` file
    let configuration = get_configuration().expect("Failed to read configuration");
    let Settings {
        application,
        database,
        email_client,
    } = configuration;

    // `actix-web` creates a worker per CPU core. Workers utilize connections
    // which are taken from the connection pool instead of creating a connection per
    // request, as an optimization technique
    let connection_pool = PgPool::connect_lazy_with(database.connect_options());
    let address = format!("{}:{}", application.host, application.port);
    let listener = TcpListener::bind(address).expect("Failed to bind address");

    let sender_email = email_client.sender().expect("Invalid sender email address");

    // WARN: `email_client` is shadowed
    let email_client = EmailClient::new(sender_email, email_client.smtp_password);

    run(listener, connection_pool, email_client)?.await?;
    Ok(())
}
