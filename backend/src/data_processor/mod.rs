// File: src/data_processing/mod.rs

pub mod api_client;
pub mod convert_html;
pub mod metadata;
pub mod data_sync;

pub use convert_html::html_to_markdown;

use crate::data_processor::api_client::ApiClient;
use crate::db::DbPool;

use crate::services::{AIModel, EmbeddingService};

use anyhow::Result;
use std::sync::Arc;
use log::info;

pub struct DataProcessor {
    pub api_client: ApiClient,
    db_pool: Arc<DbPool>,
    ai_model: Arc<AIModel>,
    embedding_service: Arc<EmbeddingService>,
}

impl DataProcessor {
    pub async fn new(db_pool: Arc<DbPool>) -> Result<Self> {
        let api_client = ApiClient::new(None, None).map_err(|e| anyhow::anyhow!("{}", e))?;
        let ai_model = Arc::new(AIModel::new());
        let embedding_service = Arc::new(EmbeddingService::new());
        
        info!("DataProcessor initialization complete");
        Ok(Self {
            api_client,
            db_pool,
            ai_model,
            embedding_service
        })
    }

    
}
