use actix_web::{post, web, HttpResponse, Responder};
use serde::Deserialize;
use std::sync::Arc;

use crate::{db::DbPool, services::search_service::SearchService};

#[derive(Deserialize)]
struct SearchQuery {
    query: String,
}

#[post("/search")]
async fn search(
    query: web::Json<SearchQuery>,
    search_service: web::Data<Arc<SearchService>>,
) -> impl Responder {
    match search_service.combined_search(query.query.clone()).await {
        Ok(results) => HttpResponse::Ok().json(results),
        Err(e) => HttpResponse::InternalServerError().body(format!("Search failed: {}", e)),
    }
}