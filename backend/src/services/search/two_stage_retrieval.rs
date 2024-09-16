use log::info;
use std::collections::HashMap;
use uuid::Uuid;

use super::SearchService;
use crate::models::Article;

impl SearchService {
    pub async fn two_stage_retrieval(
        &self,
        query: &str,
    ) -> Result<Vec<Article>, Box<dyn std::error::Error + Send + Sync>> {
        info!("Starting two-stage retrieval for query: {}", query);

        // Stage 1: Semantic search
        info!("Stage 1: Performing semantic search");
        let query_embedding = self.embedding_service.generate_embedding(query).await?;
        let mut conn = self.db_pool.get()?;
        let semantic_results =
            Article::find_relevant_articles(&query_embedding.into(), &mut conn).await?;
        info!("Semantic search found {} results", semantic_results.len());

        // Stage 2: Keyword search on semantic results
        info!("Stage 2: Performing keyword search on semantic results");
        let semantic_ids: Vec<Uuid> = semantic_results
            .iter()
            .map(|(article, _)| article.id)
            .collect();
        let keyword_results = Article::keyword_search(&mut conn, query, Some(&semantic_ids))?;
        info!("Keyword search found {} results", keyword_results.len());

        // Combine and rank results
        let mut combined_results = HashMap::new();
        for (article, score) in semantic_results {
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
        // let top_results: Vec<String> = final_results.into_iter()
        //     .take(5)
        //     .map(|(article, _)| article.markdown_content.unwrap_or(article.title))
        //     .collect();
        let top_results: Vec<Article> = final_results
            .into_iter()
            .take(5)
            .map(|(article, _)| article)
            .collect();

        info!(
            "Two-stage retrieval completed, returning {} top results",
            top_results.len()
        );
        Ok(top_results)
    }
}
