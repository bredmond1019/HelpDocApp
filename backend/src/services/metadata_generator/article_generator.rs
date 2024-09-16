use anyhow::{Context, Result};
use futures::stream::{self, StreamExt};
use log::{error, info};
use std::sync::Arc;
use tokio::sync::Semaphore;

use super::MetadataGenerator;
use crate::models::Article;

impl MetadataGenerator {
    pub async fn generate_metadata_articles(&self, processed_ids: Vec<uuid::Uuid>) -> Result<()> {
        info!("Starting article metadata generation");
        let mut conn = self
            .db_pool
            .get()
            .context("Failed to get database connection")?;
        let articles = Article::load_all(&mut conn).context("Failed to load articles")?;
        let total_articles = articles.len();

        info!("Loaded {} articles for processing", total_articles);

        let semaphore = Arc::new(Semaphore::new(self.concurrency_limit));
        let mut processed_count = 0;

        let results = stream::iter(articles)
            .filter_map(|article| {
                let data_processor = Arc::clone(&self.data_processor);
                let sem = Arc::clone(&semaphore);
                let processed_ids = processed_ids.clone();
                async move {
                    if !processed_ids.contains(&article.id) {
                        Some((article, data_processor, sem))
                    } else {
                        None
                    }
                }
            })
            .map(|(article, data_processor, sem)| async move {
                let _permit = sem.acquire().await.expect("Semaphore should not be closed");
                let result = data_processor.process_article_metadata(&article).await;
                (article.id, result)
            })
            .buffer_unordered(self.concurrency_limit)
            .inspect(|(article_id, result)| {
                processed_count += 1;
                match result {
                    Ok(_) => info!(
                        "({}/{}) Successfully updated metadata for article {}",
                        processed_count, total_articles, article_id
                    ),
                    Err(e) => error!(
                        "({}/{}) Error updating metadata for article {}: {}",
                        processed_count, total_articles, article_id, e
                    ),
                }
                self.save_checkpoint(Some(article_id.clone()), None);
            })
            .collect::<Vec<_>>()
            .await;

        for (article_id, result) in results {
            if let Err(e) = result {
                error!("Failed to store Article: {}", article_id);
                error!("Error details: {:#?}", e);
            }
        }

        info!("Completed article metadata generation");
        Ok(())
    }
}
