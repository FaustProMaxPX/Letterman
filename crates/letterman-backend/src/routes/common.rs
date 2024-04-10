use actix_web::{get, HttpResponse};
use log::info;

#[get("api/ping")]
async fn ping() -> HttpResponse {
    info!("receive ping");
    HttpResponse::Ok().json("pong")
}
