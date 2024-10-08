use std::sync::Arc;

use actix_web::{get, web, HttpResponse, Responder};
use log::error;
use serde_json::json;

use crate::services::MetadataGenerator;

#[get("/metadata-generation")]
async fn metadata_generation(
    metadata_service: web::Data<Arc<MetadataGenerator>>,
) -> impl Responder {
    let metadata_service = metadata_service.clone();
    tokio::spawn(async move {
        if let Err(e) = metadata_service.generate_article_metadata(30).await {
            error!("Error generating metadata: {}", e);
        }
    });

    HttpResponse::Accepted().json(json!({
        "message": "Generating metadata",
    }))
}

#[get("/failed-articles-metadata-generation")]
async fn failed_articles_metadata_generation(
    metadata_service: web::Data<Arc<MetadataGenerator>>,
) -> impl Responder {
    let metadata_service = metadata_service.clone();
    tokio::spawn(async move {
        if let Err(e) = metadata_service.generate_failed_article_metadata().await {
            error!("Error generating metadata: {}", e);
        }
    });

    HttpResponse::Accepted().json(json!({
        "message": "Generating metadata",
    }))
}
