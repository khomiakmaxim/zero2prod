use reqwest::Client;
use secrecy::Secret;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::net::TcpListener;

use zero2prod::{
    configuration::{get_configuration, DatabaseSettings, Settings},
    email_client::EmailClient,
    startup::run,
    telemetry::{get_subscriber, init_subscriber},
};

use std::sync::LazyLock;
static TRACING: LazyLock<()> = LazyLock::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();

    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    }
});

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

#[tokio::test]
async fn health_check_works() {
    let TestApp { address, .. } = spawn_app().await;
    let url = format!("{address}/health_check");
    let client = reqwest::Client::new();

    let response = client
        .get(url)
        .send()
        .await
        .expect("Failed to execute request");

    assert!(response.status().is_success());
    assert_eq!(response.content_length(), Some(0));
}

#[tokio::test]
async fn subscribe_returns_a_200_when_fields_are_present_but_empty() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=&email=ursula_le_guin%40gmail.com", "empty name"),
        ("name=Ursula&email=", "empty email"),
        ("name=Ursula&email=definitely-not-an-email", "invalid email"),
    ];

    for (body, description) in test_cases {
        let response = client
            .post(format!("{}/subscriptions", &app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request.");

        assert_eq!(
            400,
            response.status().as_u16(),
            "The API didn't return a 400 Bad Reqeust when the payload was {}",
            description
        );
    }
}

#[tokio::test]
async fn subscribe_returns_200_for_valid_form_data() {
    // Arrange
    let TestApp { address, db_pool } = spawn_app().await;
    let client = Client::new();

    // Act
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let response = client
        .post(format!("{}/subscriptions", &address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    // TODO: Why not to print the error specifics here?
    assert_eq!(200, response.status().as_u16());
    let saved = sqlx::query!("SELECT email, name FROM subscriptions") // why this relies on DATABASE_URL, if it already has a connection_string?
        .fetch_one(&db_pool)
        .await
        .expect("Failed to fetch saved subscription");

    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");
}

#[tokio::test]
async fn subscribe_returns_400_when_data_is_missing() {
    let TestApp { address, .. } = spawn_app().await;
    let client = Client::new();
    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        let response = client
            .post(format!("{}/subscriptions", &address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request");

        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_message
        );
    }
}

async fn spawn_app() -> TestApp {
    // The first time `initialize` is invoked the code in `TRACING` is executed.
    // All other invocations wilil instead skip execution.
    LazyLock::force(&TRACING);

    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener
        .local_addr()
        .expect("Failed to get local address")
        .port();

    let configuration = get_configuration().expect("Failed to get configuration");
    let Settings {
        mut database,
        application: _,
        email_client,
    } = configuration;
    database.database_name = uuid::Uuid::new_v4().to_string();

    let db_pool = configure_database(&database).await;
    let sender_email = email_client.sender().expect("Invalid sender email address");

    let email_client = EmailClient::new(sender_email, email_client.smtp_password, email_client.authorization_token);

    let server = run(listener, db_pool.clone(), email_client).expect("Failed to spawn app");
    tokio::task::spawn(server);
    let address = format!("http://127.0.0.1:{}", port);

    TestApp { address, db_pool }
}

pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
    // Create database
    let maintenance_settings = DatabaseSettings {
        database_name: "postgres".to_string(),
        username: "postgres".to_string(),
        password: Secret::new("password".to_string()),
        ..config.clone()
    };

    let mut connection = PgConnection::connect_with(&maintenance_settings.connect_options())
        .await
        .expect("Failed to connect to Postgres");

    connection
        .execute(format!(r#"CREATE DATABASE "{}""#, config.database_name).as_str())
        .await
        .expect("Failed to create datatbase");

    let connection_pool = PgPool::connect_with(config.connect_options())
        .await
        .expect("Failed to create Postgres db pool");

    // Migrate database
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate database");

    connection_pool
}
