use anyhow::Result;
use std::sync::Arc;

use super::data_processor::DataProcessor;
use crate::db::{init_pool, DbPool};
pub mod article_generator;
pub mod failed_article_generator;

pub struct MetadataGenerator {
    pub data_processor: Arc<DataProcessor>,
    pub db_pool: Arc<DbPool>,
    concurrency_limit: usize,
}

impl MetadataGenerator {
    pub async fn new(concurrency_limit: usize) -> Result<Self> {
        let db_pool = Arc::new(init_pool());
        let data_processor = Arc::new(DataProcessor::new(db_pool.clone()).await?);

        Ok(Self {
            data_processor,
            db_pool,
            concurrency_limit,
        })
    }
}
