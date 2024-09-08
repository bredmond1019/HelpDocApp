use std::sync::Arc;

use actix_web::{get, web, HttpResponse, Responder};
use uuid::Uuid;

use crate::jobs::JobQueue;

#[get("/job/{job_id}/status")]
pub async fn get_job_status(
    job_queue: web::Data<Arc<JobQueue>>,
    job_id: web::Path<Uuid>,
) -> impl Responder {
    let job_id = job_id.into_inner();
    match job_queue.get_job_status(job_id) {
        Some(status) => HttpResponse::Ok().json(serde_json::json!({
            "job_id": job_id,
            "status": format!("{:?}", status)
        })),
        None => HttpResponse::NotFound().body("Job not found"),
    }
}
