use std::net::TcpListener;
use zero2prod::configuration::{get_configuration, Settings};
use zero2prod::startup::run;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let configuration = get_configuration().expect("Failed to read configuration");
    let Settings {
        application_port, ..
    } = configuration;

    let address = format!("127.0.0.1:{}", application_port);
    let listener = TcpListener::bind(address).expect("Failed to bind address");

    run(listener)?.await
}
