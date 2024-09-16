use crate::{errors::SyncError, services::DataProcessor};
use actix_web::{post, web::Data, HttpResponse};
use log::info;
use serde_json::json;
use std::sync::Arc;

#[post("/parse")]
pub async fn parse_data(data_processor: Data<Arc<DataProcessor>>) -> HttpResponse {
    tokio::spawn(start_job_queue(data_processor));

    HttpResponse::Accepted().json(json!({
        "message": "Job queue started successfully",
        "status": "processing"
    }))
}

async fn start_job_queue(data_processor: Data<Arc<DataProcessor>>) -> Result<(), SyncError> {
    info!("Starting job queue");
    let collections = data_processor
        .api_client
        .get_list_collections()
        .await
        .map_err(SyncError::CollectionFetchError)?;

    for collection in collections {
        data_processor
            .prepare_sync_collection(&collection)
            .await
            .map_err(|e| SyncError::JobPreparationError {
                collection_id: collection.id.to_string(),
                error: e,
            })?;
    }

    Ok(())
}
