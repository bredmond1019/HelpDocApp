use std::{collections::HashMap, sync::Arc};

use log::info;
use tokio::task;
use uuid::Uuid;

use crate::{db::DbPool, models::Article};

use super::{AIModel, EmbeddingService};

pub struct SearchService {
    embedding_service: Arc<EmbeddingService>,
    db_pool: Arc<DbPool>,
    ai_model: Arc<AIModel>,
}

impl SearchService {
    pub fn new(db_pool: Arc<DbPool>, ai_model: Arc<AIModel>) -> Self {
        SearchService { 
            embedding_service: Arc::new(EmbeddingService::new()), 
            db_pool,
            ai_model,
        }
    }

    pub async fn combined_search(
        &self,
        query: String,
    ) -> Result<Vec<Article>, Box<dyn std::error::Error + Send + Sync>> {
        let keyword_search = task::spawn({
            let pool = self.db_pool.clone();
            let query = query.clone();
            async move {
                let mut conn = pool.get().expect("couldn't get db connection from pool");
                Article::keyword_search(&mut conn, &query)
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
            info!("Semantic result: id={}, title={}", article.id, article.title);
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
    
        Ok(combined_results.into_iter().map(|(article, _)| article).collect())
    }

    pub async fn two_stage_retrieval(
        &self,
        query: &str,
    ) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
        // Stage 1: Semantic search
        let query_embedding = self.embedding_service.generate_embedding(query).await?;
        let mut conn = self.db_pool.get()?;
        let semantic_results = Article::find_relevant_articles(&query_embedding.into(), &mut conn).await?;
 
        // Stage 2: Keyword search on semantic results
        let semantic_ids: Vec<Uuid> = semantic_results.iter().map(|(article, _)| article.id).collect();
        let keyword_results = Article::keyword_search_with_ids(&mut conn, query, &semantic_ids)?;
 
        // Combine and rank results
        let mut combined_results = HashMap::new();
        for (article, score) in semantic_results {
            combined_results.entry(article.id).or_insert((article, 0.0)).1 += score;
        }
        for article in keyword_results {
            combined_results.entry(article.id).or_insert((article, 0.0)).1 += 1.0;
        }
 
        // Sort and select top results
        let mut final_results: Vec<_> = combined_results.into_values().collect();
        final_results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        let top_results: Vec<String> = final_results.into_iter()
            .take(5)
            .map(|(article, _)| article.markdown_content.unwrap_or(article.title))
            .collect();
 
        Ok(top_results)
    }

    pub async fn expand_query(&self, query: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let ai_query_instructions = format!(
            "Expand this query with relevant keywords and phrases to improve search results. Separate terms with commas.
            Keep it short and concise.
            Query: {}", query
        );

        let ai_response = self.ai_model.generate_response(ai_query_instructions).await?;
 
        let expanded_query = ai_response.trim().to_string();
 
        Ok(format!("{}, {}", query, expanded_query))
    }
}

