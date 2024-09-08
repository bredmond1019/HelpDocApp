use actix_web::web;

pub mod job;
pub mod parse;

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(parse::parse_data);
    cfg.service(parse::get_collections);
    cfg.service(job::get_job_status);
}

use actix_web::{get, HttpResponse, Responder};

#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok().body("Welcome to the backend API. It's working!")
}
