use crate::{data_processing::fetcher::ApiClient, errors::SyncError, jobs::JobQueue};
use actix_web::{post, web::Data, HttpResponse};
use serde_json::json;
use std::sync::Arc;

#[post("/parse")]
pub async fn parse_data(job_queue: Data<Arc<JobQueue>>) -> HttpResponse {
    match start_job_queue(job_queue).await {
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

async fn start_job_queue(job_queue: Data<Arc<JobQueue>>) -> Result<Vec<String>, SyncError> {
    let api_client = ApiClient::new(None, None).map_err(SyncError::ApiClientError)?;
    let collections = api_client
        .get_list_collections()
        .await
        .map_err(SyncError::CollectionFetchError)?;

    let mut job_ids = Vec::new();

    for collection in collections {
        job_ids.push(
            job_queue
                .enqueue_sync_collection_job(collection.clone())
                .await
                .map_err(SyncError::JobEnqueueError)?,
        );

        let article_refs = ApiClient::new(None, None)
            .map_err(SyncError::ApiClientError)?
            .get_list_articles(&collection)
            .await
            .map_err(|e| SyncError::ArticleFetchError {
                collection_id: collection.id.to_string(),
                error: e.into(),
            })?;

        for article_ref in article_refs {
            job_ids.push(
                job_queue
                    .enqueue_sync_article_job(article_ref, collection.clone())
                    .await
                    .map_err(SyncError::JobEnqueueError)?,
            );
        }
    }

    Ok(job_ids)
}
