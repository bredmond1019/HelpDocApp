use actix_web::{get, post, web, HttpResponse, Responder};
use reqwest::Client;

pub mod ai_generation;
pub mod embed;
pub mod job;
pub mod parse;
pub mod search;
pub mod ws;

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(index);
    cfg.service(health);
    cfg.service(test_embed);
    cfg.service(parse::parse_data);
    cfg.service(job::get_job_status);
    cfg.service(embed::generate_embeddings);
    cfg.service(embed::get_failed_embedding_articles);
    cfg.service(embed::reembed_all_articles);
    cfg.service(search::search);
    cfg.service(ai_generation::metadata_generation);
}

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
                let body = resp
                    .text()
                    .await
                    .unwrap_or_else(|_| "Unable to read response body".to_string());
                log::error!(
                    "Embedding service returned non-success status: {}. Body: {}",
                    status,
                    body
                );
                HttpResponse::InternalServerError()
                    .body(format!("Embedding service error: Status {}", status))
            }
        }
        Err(e) => {
            log::error!("Failed to connect to embedding service: {}", e);
            HttpResponse::InternalServerError()
                .body(format!("Failed to connect to embedding service: {}", e))
        }
    }
}

#[get("/test-embed")]
pub async fn test_embed() -> impl Responder {
    let client = Client::new();
    let resp = client.get("http://localhost:8080/test-embed").send().await;
    match resp {
        Ok(resp) => {
            if resp.status().is_success() {
                HttpResponse::Ok().body(resp.text().await.unwrap())
            } else {
                HttpResponse::InternalServerError().body("Test embed failed")
            }
        }
        Err(e) => {
            log::error!("Failed to connect to embedding service: {}", e);
            HttpResponse::InternalServerError()
                .body(format!("Failed to connect to embedding service: {}", e))
        }
    }
}
