use crate::{
    data_processing::{fetcher::ApiClient, sync_processor::SyncProcessor},
    jobs::JobQueue,
};
use actix_web::{
    get, post,
    web::{self, Data},
    HttpResponse, Responder,
};
use serde_json::json;
use std::sync::Arc;

#[post("/sync")]
async fn sync_handler(job_queue: web::Data<Arc<JobQueue>>) -> impl Responder {
    let collections = match ApiClient::new(None, None).get_list_collections().await {
        Ok(collections) => collections,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    let mut job_ids = Vec::new();

    for collection in collections {
        match job_queue.enqueue_sync_collection_job(collection).await {
            Ok(job_id) => job_ids.push(job_id),
            Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
        }
    }

    HttpResponse::Accepted().json(serde_json::json!({
        "message": "Sync jobs have been queued",
        "job_ids": job_ids
    }))
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

pub async fn start_sync(job_queue: Data<JobQueue>) -> HttpResponse {
    let collections = match ApiClient::new(None, None).get_list_collections().await {
        Ok(collections) => collections,
        Err(e) => {
            return HttpResponse::InternalServerError().json(json!({ "error": e.to_string() }))
        }
    };

    let mut job_ids = Vec::new();

    for collection in collections {
        match job_queue
            .enqueue_sync_collection_job(collection.clone())
            .await
        {
            Ok(job_id) => job_ids.push(job_id),
            Err(e) => {
                return HttpResponse::InternalServerError().json(json!({ "error": e.to_string() }))
            }
        }

        let article_refs = match ApiClient::new(None, None)
            .get_list_articles(&collection)
            .await
        {
            Ok(refs) => refs,
            Err(e) => {
                return HttpResponse::InternalServerError().json(json!({ "error": e.to_string() }))
            }
        };

        for article_ref in article_refs {
            match job_queue
                .enqueue_sync_article_job(article_ref, collection.clone())
                .await
            {
                Ok(job_id) => job_ids.push(job_id),
                Err(e) => {
                    return HttpResponse::InternalServerError()
                        .json(json!({ "error": e.to_string() }))
                }
            }
        }
    }

    HttpResponse::Ok().json(json!({ "job_ids": job_ids }))
}
