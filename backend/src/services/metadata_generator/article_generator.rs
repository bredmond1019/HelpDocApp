use std::{error::Error, sync::Arc};

use anyhow::{Context, Result};
use futures::{
    lock::Mutex,
    stream::{self, StreamExt},
};
use log::{error, info, warn};
use tokio::sync::Semaphore;
use uuid::Uuid;

use super::MetadataGenerator;

use crate::models::Article;

impl MetadataGenerator {
    pub async fn generate_article_metadata(
        &self,
        limit: usize,
    ) -> Result<(Vec<Uuid>, Vec<Uuid>), Box<dyn Error + Send + Sync>> {
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
                    info!(
                        "Acquiring semaphore permit for article: {} (ID: {})",
                        article.title, article.id
                    );

                    // Acquire semaphore permit
                    let _permit = sem.acquire().await.expect("Semaphore should not be closed");

                    info!(
                        "Processing metadata for article: {} (ID: {})",
                        article.title, article.id
                    );
                    let result = data_processor.process_article_metadata(&article).await;
                    let mut count = processed_count.lock().await;
                    *count += 1;
                    (article.id, article.title.clone(), result, *count)
                }
            })
            .buffer_unordered(self.concurrency_limit)
            .collect::<Vec<_>>()
            .await;

        let mut successful_ids = Vec::new();
        let mut failed_ids = Vec::new();

        for (article_id, title, result, count) in results {
            match result {
                Ok(process_result) => {
                    if process_result.is_complete() {
                        info!(
                            "({}/{}) Successfully processed metadata for article: {} (ID: {})",
                            count, total_articles, title, article_id
                        );
                        successful_ids.push(article_id);
                    } else {
                        warn!(
                            "({}/{}) Partially processed metadata for article: {} (ID: {})",
                            count, total_articles, title, article_id
                        );
                        failed_ids.push(article_id);
                    }
                }
                Err(e) => {
                    error!(
                        "({}/{}) Error processing metadata for article {} (ID: {}): {}",
                        count, total_articles, title, article_id, e
                    );
                    failed_ids.push(article_id);
                }
            }
        }

        info!(
                "Completed article metadata generation. Processed {} articles. Successful: {}, Failed: {}",
                total_articles,
                successful_ids.len(),
                failed_ids.len()
            );

        Ok((successful_ids, failed_ids))
    }
}
