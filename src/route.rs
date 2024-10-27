use actix_web::{post, web, HttpResponse, Responder};
use serde::Deserialize;
use crate::bot;

#[derive(Deserialize)]
struct InitRequest {
    user_id: u64,
}

#[post("/init")]
async fn init_user_channel(init_request: web::Json<InitRequest>) -> impl Responder {
    // Call the bot's function to create a user channel
    match bot::create_user_channel(init_request.user_id).await {
        Ok(_) => HttpResponse::Ok().json("Channel created"),
        Err(e) => HttpResponse::InternalServerError().json(format!("Error: {:?}", e)),
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(init_user_channel);
}