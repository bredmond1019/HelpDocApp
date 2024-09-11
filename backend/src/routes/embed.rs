// routes.rs

use std::sync::Arc;

use diesel::prelude::*;
use actix_web::{post, get, web, HttpResponse, Responder};
use serde_json::json;
use log::{info, error};
use futures::stream::{self, StreamExt};

use crate::errors::SyncError;
use crate::models::Embedding;
use crate::{services::embedding_service::EmbeddingService, db::DbPool};
use crate::models::article::Article;
use crate::schema::articles;

#[post("/generate-embeddings")]
pub async fn generate_embeddings(
    pool: web::Data<Arc<DbPool>>,
) -> impl Responder {
    tokio::spawn(generate_all_embeddings(pool));
    
    HttpResponse::Accepted().json(json!({
        "message": "Embedding generation process started",
        "status": "processing"
    }))
}

#[get("/get-failed-embeddings")]
pub async fn get_failed_embedding_articles(
    pool: web::Data<Arc<DbPool>>,
) -> impl Responder {
    let failed_articles = check_failed_embeddings(pool)
        .map_err(|e| SyncError::EmbeddingError(anyhow::anyhow!("Failed to get failed embeddings: {}", e)))
        .expect("Failed to get failed embeddings");
    HttpResponse::Ok().json(json!({
        "articles": failed_articles,
        "status": "success"
    }))
}

#[post("/reembed-all")]
pub async fn reembed_all_articles(
    pool: web::Data<Arc<DbPool>>,
) -> impl Responder {
    let embedding_service = EmbeddingService::new();
    let mut conn = pool.get().expect("couldn't get db connection from pool");

    tokio::spawn(async move {
        if let Err(e) = embedding_service.reembed_all_articles(&mut conn).await {
            error!("Error re-embedding articles: {}", e);
        }
    });

    HttpResponse::Ok().json(json!({
        "message": "Started re-embedding all articles",
        "status": "success"
    }))
}


async fn generate_all_embeddings(
    pool: web::Data<Arc<DbPool>>,
) -> Result<(), SyncError> {
    let mut conn = pool.get().expect("couldn't get db connection from pool");
    let embedding_service = EmbeddingService::new();

    match articles::table.load::<Article>(&mut conn) {
        Ok(article_list) => {
            info!("Successfully retrieved {} articles", article_list.len());
            let mut success_count = 0;
            let mut error_count = 0;
            let article_count = article_list.len();

            // Process articles in batches of 50
            let results = stream::iter(article_list)
                .chunks(50)
                .map(|chunk| async {
                    let mut batch_success = 0;
                    let mut batch_error = 0;
                    for article in chunk {
                        let mut conn = pool.get().expect("couldn't get db connection from pool");
                        match embedding_service.generate_and_store_embedding(&mut conn, &article).await {
                            Ok(_) => {
                                batch_success += 1;
                                info!("Successfully generated and stored embedding for article {}", article.id);
                            }
                            Err(e) => {
                                batch_error += 1;
                                error!("Failed to generate/store embedding for article {}: {}", article.id, e);
                            }
                        }
                    }
                    (batch_success, batch_error)
                })
                .buffer_unordered(4) 
                .collect::<Vec<_>>()
                .await;

            for (batch_success, batch_error) in results {
                success_count += batch_success;
                error_count += batch_error;
            }

            info!("Embedding generation process completed: total_articles: {}, successful: {}, failed: {}",
                article_count,
                success_count,
                error_count
            );
            Ok(())
        }
        Err(e) => {
            error!("Error fetching articles: {:?}", e);
            Err(SyncError::EmbeddingError(anyhow::anyhow!("Failed to fetch articles: {}", e)))
        }
    }
}


fn check_failed_embeddings(pool: web::Data<Arc<DbPool>>) -> Result<Vec<Article>, SyncError> {
    let mut conn = pool.get().expect("couldn't get db connection from pool");
    let failed_embeddings = Embedding::get_failed_embeddings(&mut conn)
        .map_err(|e| SyncError::EmbeddingError(anyhow::anyhow!("Failed to get failed embeddings: {}", e)))?;
    info!("Found {} failed embeddings", failed_embeddings.len());

    let mut failed_articles = Vec::new();

    for embedding in failed_embeddings {
        let article = Article::get_by_id(&mut conn, embedding.article_id)
            .map_err(|e| SyncError::EmbeddingError(anyhow::anyhow!("Failed to get article: {}", e)))?;
        if let Some(article) = article {
            println!("Article: {:?}", article);
            failed_articles.push(article);
        }
    }

    Ok(failed_articles)
}