// routes.rs

use diesel::prelude::*;
use actix_web::{post, web, HttpResponse, Responder};
use serde_json::json;
use log::{info, error};
use futures::stream::{self, StreamExt};

use crate::errors::SyncError;
use crate::{data_processor::generate_and_store_embedding, db::DbPool};
use crate::models::article::Article;
use crate::schema::articles;

#[post("/generate-embeddings")]
pub async fn generate_embeddings(
    pool: web::Data<DbPool>,
) -> impl Responder {
    tokio::spawn(generate_all_embeddings(pool));
    
    HttpResponse::Accepted().json(json!({
        "message": "Embedding generation process started",
        "status": "processing"
    }))
}

async fn generate_all_embeddings(
    pool: web::Data<DbPool>,
) -> Result<(), SyncError> {
    let mut conn = pool.get().expect("couldn't get db connection from pool");

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
                        let markdown_content = article.markdown_content.or(article.html_content).unwrap_or_else(|| article.title.clone());
                        let mut conn = pool.get().expect("couldn't get db connection from pool");
                        match generate_and_store_embedding(&mut conn, article.id, &markdown_content).await {
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
                .buffer_unordered(4) // Process up to 4 batches concurrently
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