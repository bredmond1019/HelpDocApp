use std::sync::Arc;

use log::info;
use serde::{Deserialize, Serialize};

use crate::db::DbPool;

use super::{AIService, EmbeddingService};

pub mod collection_search;
pub mod combined_search;
pub mod two_stage_retrieval;

pub struct SearchService {
    embedding_service: Arc<EmbeddingService>,
    db_pool: Arc<DbPool>,
    ai_service: Arc<AIService>,
}

impl SearchService {
    pub fn new(db_pool: Arc<DbPool>, ai_service: Arc<AIService>) -> Self {
        SearchService {
            embedding_service: Arc::new(EmbeddingService::new()),
            db_pool,
            ai_service,
        }
    }

    pub async fn expand_query(
        &self,
        query: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        info!("Expanding query: {}", query);
        let ai_query_instructions = format!(
            "Expand this query with relevant keywords and phrases to improve search results. Separate terms with commas.
            Keep it short and concise.
            Query: {}", query
        );

        let ai_response = self
            .ai_service
            .generate_response(ai_query_instructions)
            .await?;

        let expanded_query = ai_response.trim().to_string();

        info!("Query expanded to: {}", expanded_query);
        Ok(format!("{}, {}", query, expanded_query))
    }
}

#[derive(Deserialize)]
pub struct SearchQuery {
    pub query: String,
}

#[derive(Serialize)]
pub struct SearchResult {
    pub articles: Vec<ArticleResult>,
    pub expanded_query: String,
}

#[derive(Serialize)]
pub struct ArticleResult {
    pub id: uuid::Uuid,
    pub title: String,
    pub content: String,
    pub slug: String,
}
