use actix_web::{post, web, HttpResponse, Responder};
use std::sync::Arc;
use crate::data_processing::sync_processor::SyncProcessor;

#[post("/sync")]
async fn sync_handler(sync_processor: web::Data<Arc<SyncProcessor>>) -> impl Responder {
    match sync_processor.sync_all().await {
        Ok(_) => HttpResponse::Ok().body("Data synchronization completed successfully"),
        Err(e) => HttpResponse::InternalServerError().body(format!("Sync error: {}", e)),
    }
}

