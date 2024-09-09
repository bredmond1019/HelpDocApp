use crate::{data_processor::DataProcessor, errors::SyncError, job::{Job, JobQueue}};
use actix_web::{post, web::Data, HttpResponse};
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

#[post("/parse")]
pub async fn parse_data(
    job_queue: Data<Arc<JobQueue>>,
    data_processor: Data<Arc<DataProcessor>>,
) -> HttpResponse {
    tokio::spawn(start_job_queue(job_queue, data_processor));

    HttpResponse::Accepted().json(json!({
        "message": "Job queue started successfully",
        "status": "processing"
    }))
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

    // Fetch collections from the database
    // let collections = Collection::get_all(&mut data_processor.db_pool.get().expect("Failed to get DB connection"))?;

    let mut job_ids = Vec::new();

    for collection in collections {
        let jobs = data_processor
            .prepare_sync_collection(&collection)
            .await
            .map_err(|e| SyncError::JobPreparationError {
                collection_id: collection.id.to_string(),
                error: e,
            })?;

        // job_ids.push(
        //     job_queue
        //         .enqueue_job(Job::EnqueueJobs(jobs))
        //         .await
        //         .map_err(SyncError::JobEnqueueError)?,
        // );
    }

    Ok(job_ids)
}
