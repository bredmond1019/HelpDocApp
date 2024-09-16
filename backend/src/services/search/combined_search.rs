use super::SearchService;

use log::info;
use tokio::task;

use crate::models::Article;

impl SearchService {
    pub async fn combined_search(
        &self,
        query: String,
    ) -> Result<Vec<Article>, Box<dyn std::error::Error + Send + Sync>> {
        let keyword_search = task::spawn({
            let pool = self.db_pool.clone();
            let query = query.clone();
            async move {
                let mut conn = pool.get().expect("couldn't get db connection from pool");
                Article::keyword_search(&mut conn, &query, None)
            }
        });

        let semantic_search = task::spawn({
            let pool = self.db_pool.clone();
            let embedding_service = self.embedding_service.clone();
            async move {
                let query_embedding = embedding_service.generate_embedding(&query).await?;
                let mut conn = pool.get().expect("couldn't get db connection from pool");
                Article::find_relevant_articles(&query_embedding.into(), &mut conn).await
            }
        });

        let (keyword_results, semantic_results) = tokio::join!(keyword_search, semantic_search);

        let keyword_results = keyword_results??;
        let semantic_results = semantic_results??;

        for article in &keyword_results {
            info!("Keyword result: id={}, title={}", article.id, article.title);
        }
        for (article, _) in &semantic_results {
            info!(
                "Semantic result: id={}, title={}",
                article.id, article.title
            );
        }

        // Combine and deduplicate results
        let mut combined_results: Vec<(Article, f64)> = Vec::new();
        let mut seen_ids = std::collections::HashSet::new();

        for article in keyword_results {
            if seen_ids.insert(article.id) {
                combined_results.push((article, 1.0)); // Give keyword results a high score
            }
        }

        for (article, score) in semantic_results {
            if seen_ids.insert(article.id) {
                combined_results.push((article, score));
            }
        }

        // Sort combined results
        combined_results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        combined_results.truncate(10); // Limit to top 10 results

        Ok(combined_results
            .into_iter()
            .map(|(article, _)| article)
            .collect())
    }
}
