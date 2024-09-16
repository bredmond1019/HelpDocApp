use std::sync::Arc;

use anyhow::{Context, Result};
use futures::{
    lock::Mutex,
    stream::{self, StreamExt},
};
use log::{debug, error, info};
use tokio::sync::Semaphore;
use uuid::Uuid;

use super::MetadataGenerator;

use crate::models::Article;

impl MetadataGenerator {
    pub async fn test_generate_metadata_articles_buffer(
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
        let articles_to_process = articles.into_iter().collect::<Vec<_>>();
        let total_articles = articles_to_process.len();

        info!("Loaded {} articles for processing", total_articles);

        let semaphore = Arc::new(Semaphore::new(self.concurrency_limit));
        let processed_count = Arc::new(Mutex::new(0));
        let processed_ids = Arc::new(Mutex::new(Vec::new()));

        info!(
            "Beginning concurrent processing with concurrency limit: {}",
            self.concurrency_limit
        );

        let results = stream::iter(articles_to_process)
            .map(|article| {
                let data_processor = Arc::clone(&self.data_processor);
                let sem = Arc::clone(&semaphore);
                let processed_count = Arc::clone(&processed_count);

                async move {
                    debug!(
                        "Acquiring semaphore permit for article: {} (ID: {})",
                        article.title, article.id
                    );
                    info!(
                        "Acquiring semaphore permit for article: {} (ID: {})",
                        article.title, article.id
                    );

                    // Acquire semaphore permit
                    let _permit = sem.acquire().await.expect("Semaphore should not be closed");

                    debug!(
                        "Processing metadata for article: {} (ID: {})",
                        article.title, article.id
                    );
                    info!(
                        "Processing metadata for article: {} (ID: {})",
                        article.title, article.id
                    );
                    let result = data_processor.test_process_metadata(&article).await;
                    let mut count = processed_count.lock().await;
                    *count += 1;
                    (article.id, article.title.clone(), result, *count)
                }
            })
            .buffer_unordered(self.concurrency_limit)
            .collect::<Vec<_>>()
            .await;

        for (article_id, title, result, count) in results {
            match result {
                Ok(_) => {
                    info!(
                        "({}/{}) Successfully processed metadata for article: {} (ID: {})",
                        count, total_articles, title, article_id
                    );
                    processed_ids.lock().await.push(article_id);
                    debug!("Added article ID: {} to processed_ids", article_id);
                }
                Err(e) => {
                    error!(
                        "({}/{}) Error processing metadata for article {} (ID: {}): {}",
                        count, total_articles, title, article_id, e
                    );
                }
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
