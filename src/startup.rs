use actix_web::{dev::Server, web, App, HttpServer};
use sqlx::PgPool;
use std::net::TcpListener;
use tracing_actix_web::TracingLogger;

use crate::{email_client::EmailClient, routes};

pub fn run(
    listener: TcpListener,
    db_pool: PgPool,
    email_client: EmailClient,
) -> Result<Server, std::io::Error> {
    let db_pool = web::Data::new(db_pool);
    let email_client = web::Data::new(email_client);
    // 'server' is a Future, which starts being polled in `main`.
    // This Future is comprised of several workers, which serve requests
    // in an endless loop
    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .route("/health_check", web::get().to(routes::health_check))
            .route("/subscriptions", web::post().to(routes::subscribe))
            // `app_data` with cloned dp_pool allows to use Form<DbPool> in all existing handlers
            .app_data(db_pool.clone()) // [`Data`] behaves as `Arc` pointer, so
            // clonning is cheap
            .app_data(email_client.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}
