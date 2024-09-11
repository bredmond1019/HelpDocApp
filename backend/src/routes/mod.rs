use std::sync::Arc;

use actix_web::{post, web};

pub mod job;
pub mod parse;
pub mod embed;

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(index); 
    cfg.service(parse::parse_data);
    cfg.service(get_collections);
    cfg.service(job::get_job_status);
    cfg.service(embed::generate_embeddings);
    cfg.service(health);
    cfg.service(embed::get_failed_embedding_articles);
}

use actix_web::{get, HttpResponse, Responder};
use reqwest::Client;

use crate::data_processor::DataProcessor;

#[get("/")]
pub async fn index() -> impl Responder {
    HttpResponse::Ok().body("Welcome to the backend API. It's working!")
}

#[post("/health")]
pub async fn health() -> impl Responder {
    let client = Client::new();

    match client.get("http://localhost:8080/health").send().await {
        Ok(resp) => {
            if resp.status().is_success() {
                HttpResponse::Ok().body("Embedding service is healthy")
            } else {
                let status = resp.status();
                let body = resp.text().await.unwrap_or_else(|_| "Unable to read response body".to_string());
                log::error!("Embedding service returned non-success status: {}. Body: {}", status, body);
                HttpResponse::InternalServerError().body(format!("Embedding service error: Status {}", status))
            }
        },
        Err(e) => {
            log::error!("Failed to connect to embedding service: {}", e);
            HttpResponse::InternalServerError().body(format!("Failed to connect to embedding service: {}", e))
        }
    }
}

#[get("/collections")]
async fn get_collections(data_processor: web::Data<Arc<DataProcessor>>) -> impl Responder {
    match data_processor.api_client.get_list_collections().await {
        Ok(collections) => HttpResponse::Ok().json(collections),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}
