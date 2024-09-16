use log::info;
use std::collections::HashMap;
use uuid::Uuid;

use super::SearchService;
use crate::models::{Article, Collection};

impl SearchService {
    pub async fn collection_based_search(
        &self,
        query: &str,
    ) -> Result<Vec<Article>, Box<dyn std::error::Error + Send + Sync>> {
        info!("Starting collection-based search for query: {}", query);

        let query_embedding = self.embedding_service.generate_embedding(query).await?;
        let mut conn = self.db_pool.get()?;

        let semantic_collection_results =
            Collection::find_relevant_collection_ids(&query_embedding.clone().into(), &mut conn)?;

        let relevant_collection_ids: Vec<Uuid> = semantic_collection_results
            .iter()
            .map(|(collection_id, _)| collection_id.clone())
            .collect();

        // Get articles from relevant collections
        let semantic_article_results = Article::find_relevant_articles_by_collection_ids(
            &query_embedding.clone().into(),
            &mut conn,
            &relevant_collection_ids,
        )
        .await?;

        info!(
            "Semantic search found {} results",
            semantic_article_results.len()
        );

        let semantic_ids: Vec<Uuid> = semantic_article_results
            .iter()
            .map(|(article, _)| article.id)
            .collect();
        let keyword_results = Article::keyword_search(&mut conn, query, Some(&semantic_ids))?;
        info!("Keyword search found {} results", keyword_results.len());

        // Combine and rank results
        let mut combined_results = HashMap::new();
        for (article, score) in semantic_article_results {
            combined_results
                .entry(article.id)
                .or_insert((article, 0.0))
                .1 += score;
        }
        for article in keyword_results {
            combined_results
                .entry(article.id)
                .or_insert((article, 0.0))
                .1 += 1.0;
        }

        // Sort and select top results
        let mut final_results: Vec<_> = combined_results.into_values().collect();
        final_results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        let top_results: Vec<Article> = final_results
            .into_iter()
            .take(5)
            .map(|(article, _)| article)
            .collect();

        info!(
            "Collection-based search completed, returning {} top results",
            top_results.len()
        );
        Ok(top_results)
    }
}
