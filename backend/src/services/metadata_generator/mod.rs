use anyhow::{Context, Result};
use log::error;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::BufReader;
use std::sync::Arc;

use crate::db::{init_pool, DbPool};

use super::{
    ai::{ai_data_service::AIDataService, ollama_load_balancer::OllamaLoadBalancer},
    data_processor::DataProcessor,
};

pub mod article_generator;
pub mod collection_generator;
pub mod test_generator_balancer;
pub mod test_generator_buffer;

#[derive(Serialize, Deserialize)]
struct Checkpoint {
    processed_article_ids: Vec<uuid::Uuid>,
    processed_collection_ids: Vec<uuid::Uuid>,
}

impl Default for Checkpoint {
    fn default() -> Self {
        Checkpoint {
            processed_article_ids: Vec::new(),
            processed_collection_ids: Vec::new(),
        }
    }
}

pub struct MetadataGenerator {
    pub data_processor: Arc<DataProcessor>,
    pub db_pool: Arc<DbPool>,
    concurrency_limit: usize,
    ollama_balancer: Arc<OllamaLoadBalancer>,
}

impl MetadataGenerator {
    pub async fn new(
        concurrency_limit: usize,
        server_ports: &[u16],
        threads_per_server: usize,
    ) -> Result<Self> {
        let db_pool = Arc::new(init_pool());
        let ollama_balancer = Arc::new(OllamaLoadBalancer::new(server_ports, threads_per_server));
        let ai_data_service = Arc::new(AIDataService::new(Arc::clone(&ollama_balancer)));
        let data_processor =
            Arc::new(DataProcessor::new(db_pool.clone(), ai_data_service.clone()).await?);

        Ok(Self {
            data_processor,
            db_pool,
            concurrency_limit,
            ollama_balancer,
        })
    }

    pub async fn generate_all_metadata(&self) -> Result<()> {
        let checkpoint = self.load_checkpoint().unwrap_or_default();

        self.generate_metadata_articles(checkpoint.processed_article_ids)
            .await?;
        self.generate_metadata_collections(checkpoint.processed_collection_ids)
            .await?;

        // Clear checkpoint after successful completion
        std::fs::remove_file("metadata_checkpoint.json").ok();

        Ok(())
    }

    fn load_checkpoint(&self) -> Result<Checkpoint> {
        let file =
            File::open("metadata_checkpoint.json").context("Failed to open checkpoint file")?;
        let reader = BufReader::new(file);
        let checkpoint: Checkpoint =
            serde_json::from_reader(reader).context("Failed to parse checkpoint file")?;
        Ok(checkpoint)
    }

    fn save_checkpoint(&self, article_id: Option<uuid::Uuid>, collection_id: Option<uuid::Uuid>) {
        let mut checkpoint = self.load_checkpoint().unwrap_or_default();

        if let Some(id) = article_id {
            checkpoint.processed_article_ids.push(id);
        }
        if let Some(id) = collection_id {
            checkpoint.processed_collection_ids.push(id);
        }

        if let Err(e) = serde_json::to_writer(
            File::create("metadata_checkpoint.json").expect("Failed to create checkpoint file"),
            &checkpoint,
        ) {
            error!("Failed to save checkpoint: {}", e);
        }
    }
}
