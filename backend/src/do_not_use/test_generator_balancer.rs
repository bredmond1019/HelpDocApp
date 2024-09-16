use std::sync::Arc;

use anyhow::{Context, Result};
use futures::future::join_all;
use futures::lock::Mutex;
use log::{debug, error, info};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use tokio::runtime::Runtime;
use uuid::Uuid;

use super::MetadataGenerator;

use crate::models::Article;

impl MetadataGenerator {
    fn calculate_chunk_size(&self, total_articles: usize) -> usize {
        (total_articles + self.concurrency_limit - 1) / self.concurrency_limit
    }

    pub async fn test_generate_metadata_articles_balancer(
        &self,
        limit: usize,
    ) -> Result<Vec<Uuid>, anyhow::Error> {
        info!(
            "Starting test article metadata generation for up to {} articles",
            limit
        );
        let mut conn = self
            .db_pool
            .get()
            .context("Failed to get database connection")?;
        let articles =
            Article::load_batch(&mut conn, 0, limit).context("Failed to load articles")?;
        let articles_to_process = articles;
        let total_articles = articles_to_process.len();

        info!("Loaded {} articles for processing", total_articles);

        let processed_ids = Arc::new(Mutex::new(Vec::new()));

        info!(
            "Beginning concurrent processing with concurrency limit: {}",
            self.concurrency_limit
        );

        let chunk_size = self.calculate_chunk_size(total_articles);
        let article_chunks: Vec<Vec<Article>> = articles_to_process
            .chunks(chunk_size)
            .map(|chunk| chunk.to_vec())
            .collect();

        info!("Chunk size: {}", chunk_size);

        let futures = article_chunks.into_iter().map(|chunk| {
            let data_processor = Arc::clone(&self.data_processor);
            let processed_ids = Arc::clone(&processed_ids);
            let ollama_balancer = Arc::clone(&self.ollama_balancer);
    
            tokio::spawn(async move {
                info!("Checkpoint 1");
                ollama_balancer.execute(move || {
                    info!("Checkpoint 2");
                    chunk
                        .into_par_iter()
                        .map(|article| {
                            info!("Checkpoint 3");
                            let data_processor = Arc::clone(&data_processor);
                            let processed_ids = Arc::clone(&processed_ids);
    
                            // Create a new Tokio runtime for each parallel task
                            let rt = Runtime::new().unwrap();
                            rt.block_on(async {
                                info!("Checkpoint 4");
                                info!(
                                    "Processing metadata for article: {} (ID: {})",
                                    article.title, article.id
                                );
                                let result = data_processor.test_process_metadata(&article).await;
                                match &result {
                                    Ok(_) => {
                                        info!(
                                            "Successfully processed metadata for article: {} (ID: {})",
                                            article.title, article.id
                                        );
                                        processed_ids.lock().await.push(article.id);
                                    }
                                    Err(e) => {
                                        error!(
                                            "Error processing metadata for article {} (ID: {}): {}",
                                            article.title, article.id, e
                                        );
                                    }
                                }
                                result
                            })
                        })
                        .collect::<Vec<_>>()
                })
            })
        });

        info!("Waiting for all futures to complete");
        let results = join_all(futures).await;

        for result in results {
            if let Err(e) = result {
                error!("Error in processing thread: {}", e);
            }
        }

        let final_processed_ids = processed_ids.lock().await.clone();
        info!(
            "Completed test article metadata generation. Processed {} articles.",
            final_processed_ids.len()
        );
        debug!("Final processed article IDs: {:?}", final_processed_ids);

        Ok(final_processed_ids)
    }
}
