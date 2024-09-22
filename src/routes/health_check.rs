use actix_web::{HttpResponse, Responder};

pub async fn health_check() -> impl Responder {
    tracing::info!("Serving health check request");
    HttpResponse::Ok().finish()
}
