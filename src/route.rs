use actix_web::{post, HttpResponse, Responder};
use crate::bot;

#[post("/init")]
async fn init_user_channel() -> impl Responder {
    // This will eventually call bot::create_user_channel with the user's Discord ID
    HttpResponse::Ok().json("Channel initialization placeholder")
}

pub fn init(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(init_user_channel);
}