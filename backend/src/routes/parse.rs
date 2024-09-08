use crate::{data_processor::data_processor::DataProcessor, errors::SyncError, jobs::JobQueue};
use actix_web::{post, web::Data, HttpResponse};
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

#[post("/parse")]
pub async fn parse_data(
    job_queue: Data<Arc<JobQueue>>,
    data_processor: Data<Arc<DataProcessor>>,
) -> HttpResponse {
    match start_job_queue(job_queue, data_processor).await {
        Ok(job_ids) => HttpResponse::Ok().json(json!({ "job_ids": job_ids })),
        Err(e) => {
            log::error!("Sync error: {:?}", e);
            HttpResponse::InternalServerError().json(json!({
                "error": e.to_string(),
                "error_type": format!("{:?}", e)
            }))
        }
    }
}

async fn start_job_queue(
    job_queue: Data<Arc<JobQueue>>,
    data_processor: Data<Arc<DataProcessor>>,
) -> Result<Vec<Uuid>, SyncError> {
    let collections = data_processor
        .api_client
        .get_list_collections()
        .await
        .map_err(SyncError::CollectionFetchError)?;

    let mut job_ids = Vec::new();

    for collection in collections {
        let sync_jobs = data_processor
            .prepare_sync_collection(&collection)
            .await
            .map_err(|e| SyncError::JobPreparationError {
                collection_id: collection.id.to_string(),
                error: e,
            })?;

        for job in sync_jobs {
            job_ids.push(
                job_queue
                    .enqueue_job(job)
                    .await
                    .map_err(SyncError::JobEnqueueError)?,
            );
        }
    }

    Ok(job_ids)
}
