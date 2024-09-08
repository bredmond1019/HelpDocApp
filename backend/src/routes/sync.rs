use crate::{data_processing::sync_processor::SyncProcessor, jobs::JobQueue};
use actix_web::{get, post, web, HttpResponse, Responder};
use std::sync::Arc;

#[post("/sync")]
async fn sync_handler(job_queue: web::Data<Arc<JobQueue>>) -> impl Responder {
    match job_queue.enqueue_sync_job().await {
        Ok(job_id) => HttpResponse::Accepted().json(serde_json::json!({
            "message": "Sync job has been queued",
            "job_id": job_id
        })),
        Err(e) => {
            HttpResponse::InternalServerError().body(format!("Failed to queue sync job: {}", e))
        }
    }
}

#[get("/sync/status/{job_id}")]
async fn get_sync_status(
    job_queue: web::Data<Arc<JobQueue>>,
    job_id: web::Path<String>,
) -> impl Responder {
    match job_queue.get_job_status(&job_id) {
        Some(status) => HttpResponse::Ok().json(serde_json::json!({
            "job_id": job_id.into_inner(),
            "status": format!("{:?}", status)
        })),
        None => HttpResponse::NotFound().body("Job not found"),
    }
}

#[get("/collections")]
async fn get_collections(sync_processor: web::Data<Arc<SyncProcessor>>) -> impl Responder {
    match sync_processor.api_client.get_list_collections().await {
        Ok(collections) => HttpResponse::Ok().json(collections),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}
