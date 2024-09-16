// File: src/data_processing/mod.rs

use anyhow::Result;
use log::info;
use std::sync::Arc;

use crate::db::DbPool;
use crate::services::ai::ollama_load_balancer::OllamaLoadBalancer;
use crate::services::{AIService, EmbeddingService};
use api_client::ApiClient;

pub mod api_client;
pub mod convert_html;
pub mod data_sync;
pub mod metadata;

pub use convert_html::html_to_markdown;

use super::ai::ai_data_service::AIDataService;
pub struct DataProcessor {
    pub api_client: ApiClient,
    db_pool: Arc<DbPool>,
    ai_service: Arc<AIService>,
    ai_data_service: Arc<AIDataService>,
    embedding_service: Arc<EmbeddingService>,
}

impl DataProcessor {
    pub async fn new(db_pool: Arc<DbPool>, ai_data_service: Arc<AIDataService>) -> Result<Self> {
        let api_client = ApiClient::new(None, None).map_err(|e| anyhow::anyhow!("{}", e))?;
        let ai_service = Arc::new(AIService::new());
        let embedding_service = Arc::new(EmbeddingService::new());
        // let ollama_balancer = Arc::new(OllamaLoadBalancer::new(server_ports, threads_per_server));
        // let ai_data_service = Arc::new(AIDataService::new(Arc::clone(&ollama_balancer)));

        info!("DataProcessor initialization complete");
        Ok(Self {
            api_client,
            db_pool,
            ai_service,
            embedding_service,
            ai_data_service,
        })
    }
}
