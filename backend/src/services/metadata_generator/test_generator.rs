use anyhow::{Context, Result};
use log::info;

use super::MetadataGenerator;

use crate::models::Article;

impl MetadataGenerator {
    pub async fn test_generate_metadata_articles(
        &self,
        limit: usize,
    ) -> Result<Vec<String>, anyhow::Error> {
        info!(
            "Starting test article metadata generation for {} articles",
            limit
        );
        let mut conn = self
            .db_pool
            .get()
            .context("Failed to get database connection")?;
        let articles = Article::load_all(&mut conn).context("Failed to load articles")?;
        let articles_to_process = articles.into_iter().take(limit).collect::<Vec<_>>();

        let mut results = Vec::new();

        for (index, article) in articles_to_process.iter().enumerate() {
            info!(
                "Processing article {}/{}: {}",
                index + 1,
                limit,
                article.title
            );
            match self.data_processor.process_article_metadata(article).await {
                Ok(_) => {
                    let result = format!(
                        "Successfully processed metadata for article: {}",
                        article.title
                    );
                    results.push(result);
                }
                Err(e) => {
                    let error = format!(
                        "Error processing metadata for article {}: {}",
                        article.title, e
                    );
                    results.push(error);
                }
            }
        }

        Ok(results)
    }
}
