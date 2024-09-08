use crate::{
    data_processing::{data_processor::SyncProcessor, fetcher::ApiClient},
    jobs::JobQueue,
};
use actix_web::{
    get, post,
    web::{self, Data},
    HttpResponse, Responder,
};
use serde_json::json;
use std::sync::Arc;

#[post("/parse")]
async fn parse_data(job_queue: web::Data<Arc<JobQueue>>) -> impl Responder {
    let api_client = match ApiClient::new(None, None) {
        Ok(client) => client,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    let collections = match api_client.get_list_collections().await {
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

#[get("/collections")]
async fn get_collections(sync_processor: web::Data<Arc<SyncProcessor>>) -> impl Responder {
    match sync_processor.api_client.get_list_collections().await {
        Ok(collections) => HttpResponse::Ok().json(collections),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

pub async fn start_sync(job_queue: Data<JobQueue>) -> HttpResponse {
    let api_client = match ApiClient::new(None, None) {
        Ok(client) => client,
        Err(e) => {
            return HttpResponse::InternalServerError().json(json!({ "error": e.to_string() }))
        }
    };

    let collections = match api_client.get_list_collections().await {
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

        let article_refs = match ApiClient::new(None, None) {
            Ok(client) => client.get_list_articles(&collection).await,
            Err(e) => {
                return HttpResponse::InternalServerError().json(json!({ "error": e.to_string() }))
            }
        };

        let article_refs = match article_refs {
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
