use secrecy::ExposeSecret;
use sqlx::PgPool;
use std::net::TcpListener;

use zero2prod::{
    configuration::{get_configuration, Settings},
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
        application_port,
        database,
    } = configuration;

    // `actix-web` creates a worker per CPU core. Workers utilize connections
    // which are taken from the connection pool instead of creating a connection per
    // request as an optimization technique
    let connection_pool = PgPool::connect(database.connection_string().expose_secret())
        .await
        .expect("Failed to connect to Postgres");

    let address = format!("{}:{}", database.host, application_port);
    // CHECK: Why does `bind()` depend on an entire address and not just a port?
    // Can we use some other address except for localhost\127.0.0.1?
    // Maybe we can/ Something like 127.0.0.(2,3,4,x)
    //
    // UPD: Yes, we can use not only 127.0.0.1 for hosting a local service.
    // In fact, I managed to do so with 127.0.0.2 and everything worked as planned when
    // issuing requests via `curl`, yet 2 of my browsers returned `unnable to connect`.
    // It seems to be a hardcoded browsers limitation for some reason.
    let listener = TcpListener::bind(address).expect("Failed to bind address");

    run(listener, connection_pool)?.await?;
    Ok(())
}
