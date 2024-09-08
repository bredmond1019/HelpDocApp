use actix_web::web;

pub mod sync;

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(sync::sync_handler);
    cfg.service(sync::get_collections);
    cfg.service(index);
}

use actix_web::{get, HttpResponse, Responder};

#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok().body("Welcome to the backend API. It's working!")
}
