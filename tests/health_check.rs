use std::net::TcpListener;

#[tokio::test]
async fn health_check_works() {
    let ip_addr = spawn_app();
    let url = format!("http://{ip_addr}/health_check");
    println!("{url}");

    let client = reqwest::Client::new();

    let response = client
        .get(url)
        .send()
        .await
        .expect("Failed to execute request");

    assert!(response.status().is_success());
    assert_eq!(response.content_length(), Some(0));
}

fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener
        .local_addr()
        .expect("Failed to get local address")
        .port();

    println!("{port}");

    let server = zero2prod::run(listener).expect("Failed to spawn app");
    tokio::task::spawn(server);
    format!("127.0.0.1:{}", port)
}
