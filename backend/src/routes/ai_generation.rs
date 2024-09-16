use std::sync::Arc;

use actix_web::{get, web, HttpResponse, Responder};
use log::{error, info};
use serde_json::json;

use crate::services::MetadataGenerator;

#[get("/metadata-generation")]
async fn metadata_generation(
    metadata_service: web::Data<Arc<MetadataGenerator>>,
) -> impl Responder {
    tokio::spawn(async move {
        let response = metadata_service.generate_article_metadata(30).await;
        match response {
            Ok(results) => {
                info!("Metadata generated: {:?}", results);
            }
            Err(e) => error!("Error generating metadata: {}", e),
        }
    });

    HttpResponse::Accepted().json(json!({
        "message": "Generating metadata",
    }))
}
