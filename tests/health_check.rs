use reqwest::Client;
use sqlx::{Connection, PgConnection, PgPool};
use std::net::TcpListener;
use zero2prod::{
    configuration::{get_configuration, Settings},
    startup::run,
};

#[tokio::test]
async fn health_check_works() {
    let address = spawn_app().await;
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
async fn subscribe_returns_200_for_valid_form_data() {
    // Arrange
    let address = spawn_app().await;
    let client = Client::new();
    let configuration = get_configuration().expect("Failed to read configuration");
    let connection_string = configuration.database.connection_string();

    // Act
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let response = client
        .post(format!("{}/subscriptions", &address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request");

    let mut connection = PgConnection::connect(&connection_string)
        .await
        .expect("Failed to connect to Postgres");

    // Assert
    // TODO: Why not to print the error specifics here?
    assert_eq!(200, response.status().as_u16());
    let saved = sqlx::query!("SELECT email, name FROM subscriptions") // why this relies on DATABASE_URL, if it already has a connection_string?
        .fetch_one(&mut connection)
        .await
        .expect("Failed to fetch saved subscription");

    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");
}

#[tokio::test]
async fn subscribe_returns_400_when_data_is_missing() {
    let address = spawn_app().await;
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

async fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener
        .local_addr()
        .expect("Failed to get local address")
        .port();
    println!("{port}");

    let configuration = get_configuration().expect("Failed to get configuration");
    let Settings { database, .. } = configuration;

    let db_pool = PgPool::connect(&database.connection_string())
        .await
        .expect("Failed to connect to Postgres");

    let server = run(listener, db_pool).expect("Failed to spawn app");
    tokio::task::spawn(server);
    format!("http://127.0.0.1:{}", port)
}
