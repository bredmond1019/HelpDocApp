use anyhow::{Context, Result};
use futures::stream::{self, StreamExt};
use log::{error, info};
use std::sync::Arc;
use tokio::sync::Semaphore;

use super::MetadataGenerator;
use crate::models::Collection;

impl MetadataGenerator {
    pub async fn generate_metadata_collections(
        &self,
        processed_ids: Vec<uuid::Uuid>,
    ) -> Result<()> {
        info!("Starting collection metadata generation");
        let mut conn = self
            .db_pool
            .get()
            .context("Failed to get database connection")?;
        let collections = Collection::load_all(&mut conn).context("Failed to load collections")?;
        let total_collections = collections.len();

        info!("Loaded {} collections for processing", total_collections);

        let semaphore = Arc::new(Semaphore::new(self.concurrency_limit));
        let mut processed_count = 0;

        let results = stream::iter(collections)
            .filter_map(|collection| {
                let data_processor = Arc::clone(&self.data_processor);
                let sem = Arc::clone(&semaphore);
                let processed_ids = processed_ids.clone();
                async move {
                    if !processed_ids.contains(&collection.id) {
                        Some((collection, data_processor, sem))
                    } else {
                        None
                    }
                }
            })
            .map(|(collection, data_processor, sem)| async move {
                let _permit = sem.acquire().await.expect("Semaphore should not be closed");
                let result = data_processor
                    .process_collection_metadata(&collection)
                    .await;
                (collection.id, result)
            })
            .buffer_unordered(self.concurrency_limit)
            .inspect(|(collection_id, result)| {
                processed_count += 1;
                match result {
                    Ok(_) => info!(
                        "({}/{}) Successfully updated metadata for collection {}",
                        processed_count, total_collections, collection_id
                    ),
                    Err(e) => error!(
                        "({}/{}) Error updating metadata for collection {}: {}",
                        processed_count, total_collections, collection_id, e
                    ),
                }
                self.save_checkpoint(None, Some(*collection_id));
            })
            .collect::<Vec<_>>()
            .await;

        for (collection_id, result) in results {
            if let Err(e) = result {
                error!("Failed to store Collection: {}", collection_id);
                error!("Error details: {:#?}", e);
            }
        }

        info!("Completed collection metadata generation");
        Ok(())
    }
}
