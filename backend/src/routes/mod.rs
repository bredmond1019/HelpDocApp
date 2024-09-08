use std::sync::Arc;

use actix_web::web;

pub mod job;
pub mod parse;

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(parse::parse_data);
    cfg.service(get_collections);
    cfg.service(job::get_job_status);
}

use actix_web::{get, HttpResponse, Responder};

use crate::data_processor::DataProcessor;

#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok().body("Welcome to the backend API. It's working!")
}

#[get("/collections")]
async fn get_collections(data_processor: web::Data<Arc<DataProcessor>>) -> impl Responder {
    match data_processor.api_client.get_list_collections().await {
        Ok(collections) => HttpResponse::Ok().json(collections),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}
