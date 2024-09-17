// File: src/data_processing/mod.rs

use anyhow::Result;
use log::info;
use pgvector::Vector;
use std::sync::Arc;
use uuid::Uuid;

use crate::db::DbPool;
use crate::services::{AIService, EmbeddingService};
use api_client::ApiClient;

pub mod api_client;
pub mod convert_html;
pub mod data_sync;
pub mod process_article;
pub mod process_failed_articles;

pub use convert_html::html_to_markdown;

pub struct DataProcessor {
    pub api_client: ApiClient,
    db_pool: Arc<DbPool>,
    ai_service: Arc<AIService>,
    embedding_service: Arc<EmbeddingService>,
}

impl DataProcessor {
    pub async fn new(db_pool: Arc<DbPool>) -> Result<Self> {
        let api_client = ApiClient::new(None, None).map_err(|e| anyhow::anyhow!("{}", e))?;
        let ai_service = Arc::new(AIService::new());
        let embedding_service = Arc::new(EmbeddingService::new());

        info!("DataProcessor initialization complete");
        Ok(Self {
            api_client,
            db_pool,
            ai_service,
            embedding_service,
        })
    }
}

#[derive(Debug, Clone)]
pub struct ProcessResult {
    pub id: Uuid,
    pub paragraph: Option<String>,
    pub bullets: Option<Vec<Option<String>>>,
    pub keywords: Option<Vec<Option<String>>>,
    pub paragraph_description_embedding: Option<Vector>,
    pub bullet_points_embedding: Option<Vector>,
    pub keywords_embedding: Option<Vector>,
}

impl ProcessResult {
    pub fn new(id: Uuid) -> Self {
        Self {
            id,
            paragraph: None,
            bullets: None,
            keywords: None,
            paragraph_description_embedding: None,
            bullet_points_embedding: None,
            keywords_embedding: None,
        }
    }

    pub fn has_content(&self) -> bool {
        self.paragraph.is_some()
            || self.bullets.is_some()
            || self.keywords.is_some()
            || self.paragraph_description_embedding.is_some()
            || self.bullet_points_embedding.is_some()
            || self.keywords_embedding.is_some()
    }

    pub fn is_complete(&self) -> bool {
        self.paragraph.is_some() && self.bullets.is_some() && self.keywords.is_some()
    }
}
