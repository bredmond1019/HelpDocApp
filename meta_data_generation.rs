use std::sync::Arc;
use anyhow::Result;
use futures::stream::{self, StreamExt};
use crate::db::{DbPool, init_pool};
use crate::models::{Article, Collection};
use crate::data_processor::DataProcessor;

pub struct MetadataGenerator {
    data_processor: Arc<DataProcessor>,
    db_pool: Arc<DbPool>,
}

impl MetadataGenerator {
    pub async fn new() -> Result<Self> {
        let db_pool = Arc::new(init_pool());
        let data_processor = Arc::new(DataProcessor::new(db_pool.clone()).await?);

        Ok(Self {
            data_processor,
            db_pool,
        })
    }

    pub async fn generate_all_metadata(&self) -> Result<()> {
        self.generate_article_metadata().await?;
        self.generate_collection_metadata().await?;
        Ok(())
    }

    async fn generate_article_metadata(&self) -> Result<()> {
        let mut conn = self.db_pool.get()?;
        let articles = Article::load_all(&mut conn)?;

        stream::iter(articles)
            .map(|article| {
                let data_processor = Arc::clone(&self.data_processor);
                async move {
                    if let Err(e) = data_processor.update_article_metadata(&article).await {
                        eprintln!("Error updating metadata for article {}: {}", article.id, e);
                    }
                }
            })
            .buffer_unordered(4) // Process up to 4 articles concurrently
            .collect::<Vec<_>>()
            .await;

        Ok(())
    }

    async fn generate_collection_metadata(&self) -> Result<()> {
        let mut conn = self.db_pool.get()?;
        let collections = Collection::load_all(&mut conn)?;

        stream::iter(collections)
            .map(|collection| {
                let data_processor = Arc::clone(&self.data_processor);
                async move {
                    if let Err(e) = data_processor.update_collection_metadata(&collection).await {
                        eprintln!("Error updating metadata for collection {}: {}", collection.id, e);
                    }
                }
            })
            .buffer_unordered(4) // Process up to 4 collections concurrently
            .collect::<Vec<_>>()
            .await;

        Ok(())
    }
}

pub async fn run_metadata_generation() -> Result<()> {
    let generator = MetadataGenerator::new().await?;
    generator.generate_all_metadata().await
}


mod scripts;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // ... other imports and setup ...

    let cli = Cli::parse();

    match cli.command {
        // ... other commands ...
        Commands::GenerateMetadata => {
            scripts::metadata_generation::run_metadata_generation().await?;
        }
    }

    Ok(())
}




// 
// 
// OPTIMIZED FOR THREADING
use std::sync::Arc;
use anyhow::Result;
use futures::stream::{self, StreamExt};
use tokio::sync::Semaphore;
use crate::db::{DbPool, init_pool};
use crate::models::{Article, Collection};
use crate::data_processor::DataProcessor;

pub struct MetadataGenerator {
    data_processor: Arc<DataProcessor>,
    db_pool: Arc<DbPool>,
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

    pub async fn generate_all_metadata(&self) -> Result<()> {
        self.generate_article_metadata().await?;
        self.generate_collection_metadata().await?;
        Ok(())
    }

    async fn generate_article_metadata(&self) -> Result<()> {
        let mut conn = self.db_pool.get()?;
        let articles = Article::load_all(&mut conn)?;

        let semaphore = Arc::new(Semaphore::new(self.concurrency_limit));

        let results = stream::iter(articles)
            .map(|article| {
                let data_processor = Arc::clone(&self.data_processor);
                let sem = Arc::clone(&semaphore);
                async move {
                    let _permit = sem.acquire().await.expect("Semaphore should not be closed");
                    let result = data_processor.update_article_metadata(&article).await;
                    (article.id, result)
                }
            })
            .buffer_unordered(self.concurrency_limit)
            .collect::<Vec<_>>()
            .await;

        for (article_id, result) in results {
            match result {
                Ok(_) => log::info!("Successfully updated metadata for article {}", article_id),
                Err(e) => log::error!("Error updating metadata for article {}: {}", article_id, e),
            }
        }

        Ok(())
    }

    async fn generate_collection_metadata(&self) -> Result<()> {
        let mut conn = self.db_pool.get()?;
        let collections = Collection::load_all(&mut conn)?;

        let semaphore = Arc::new(Semaphore::new(self.concurrency_limit));

        let results = stream::iter(collections)
            .map(|collection| {
                let data_processor = Arc::clone(&self.data_processor);
                let sem = Arc::clone(&semaphore);
                async move {
                    let _permit = sem.acquire().await.expect("Semaphore should not be closed");
                    let result = data_processor.update_collection_metadata(&collection).await;
                    (collection.id, result)
                }
            })
            .buffer_unordered(self.concurrency_limit)
            .collect::<Vec<_>>()
            .await;

        for (collection_id, result) in results {
            match result {
                Ok(_) => log::info!("Successfully updated metadata for collection {}", collection_id),
                Err(e) => log::error!("Error updating metadata for collection {}: {}", collection_id, e),
            }
        }

        Ok(())
    }
}

pub async fn run_metadata_generation() -> Result<()> {
    let generator = MetadataGenerator::new(4).await?; // Adjust concurrency as needed
    generator.generate_all_metadata().await
}



// 
// 
// Enhanced for Error Handling and Checkpoint

use std::sync::Arc;
use anyhow::{Result, Context};
use futures::stream::{self, StreamExt};
use tokio::sync::Semaphore;
use serde::{Serialize, Deserialize};
use std::fs::File;
use std::io::BufReader;
use crate::db::{DbPool, init_pool};
use crate::models::{Article, Collection};
use crate::data_processor::DataProcessor;

#[derive(Serialize, Deserialize)]
struct Checkpoint {
    processed_article_ids: Vec<uuid::Uuid>,
    processed_collection_ids: Vec<uuid::Uuid>,
}

pub struct MetadataGenerator {
    data_processor: Arc<DataProcessor>,
    db_pool: Arc<DbPool>,
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

    pub async fn generate_all_metadata(&self) -> Result<()> {
        let checkpoint = self.load_checkpoint().unwrap_or_default();
        
        self.generate_article_metadata(checkpoint.processed_article_ids).await?;
        self.generate_collection_metadata(checkpoint.processed_collection_ids).await?;
        
        // Clear checkpoint after successful completion
        std::fs::remove_file("metadata_checkpoint.json").ok();
        
        Ok(())
    }

    async fn generate_article_metadata(&self, processed_ids: Vec<uuid::Uuid>) -> Result<()> {
        info!("Starting article metadata generation");
        let mut conn = self.db_pool.get().context("Failed to get database connection")?;
        let articles = Article::load_all(&mut conn).context("Failed to load articles")?;
        let total_articles = articles.len();
        
        info!("Loaded {} articles for processing", total_articles);

        let semaphore = Arc::new(Semaphore::new(self.concurrency_limit));
        let mut processed_count = 0;

        let results = stream::iter(articles)
            .filter(|article| !processed_ids.contains(&article.id))
            .map(|article| {
                let data_processor = Arc::clone(&self.data_processor);
                let sem = Arc::clone(&semaphore);
                async move {
                    let _permit = sem.acquire().await.expect("Semaphore should not be closed");
                    let result = data_processor.update_article_metadata(&article).await;
                    (article.id, result)
                }
            })
            .buffer_unordered(self.concurrency_limit)
            .inspect(|(article_id, result)| {
                processed_count += 1;
                match result {
                    Ok(_) => info!("({}/{}) Successfully updated metadata for article {}", processed_count, total_articles, article_id),
                    Err(e) => error!("({}/{}) Error updating metadata for article {}: {}", processed_count, total_articles, article_id, e),
                }
                self.save_checkpoint(article_id, None);
            })
            .collect::<Vec<_>>()
            .await;

        for (article_id, result) in results {
            if let Err(e) = result {
                error!("Failed to store Article: {}", article_id);
                error!("Error details: {:#?}", e);
            }
        }

        info!("Completed article metadata generation");
        Ok(())
    }

    async fn generate_collection_metadata(&self, processed_ids: Vec<uuid::Uuid>) -> Result<()> {
        info!("Starting collection metadata generation");
        let mut conn = self.db_pool.get().context("Failed to get database connection")?;
        let collections = Collection::load_all(&mut conn).context("Failed to load collections")?;
        let total_collections = collections.len();
        
        info!("Loaded {} collections for processing", total_collections);

        let semaphore = Arc::new(Semaphore::new(self.concurrency_limit));
        let mut processed_count = 0;

        let results = stream::iter(collections)
            .filter(|collection| !processed_ids.contains(&collection.id))
            .map(|collection| {
                let data_processor = Arc::clone(&self.data_processor);
                let sem = Arc::clone(&semaphore);
                async move {
                    let _permit = sem.acquire().await.expect("Semaphore should not be closed");
                    let result = data_processor.update_collection_metadata(&collection).await;
                    (collection.id, result)
                }
            })
            .buffer_unordered(self.concurrency_limit)
            .inspect(|(collection_id, result)| {
                processed_count += 1;
                match result {
                    Ok(_) => info!("({}/{}) Successfully updated metadata for collection {}", processed_count, total_collections, collection_id),
                    Err(e) => error!("({}/{}) Error updating metadata for collection {}: {}", processed_count, total_collections, collection_id, e),
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

    fn load_checkpoint(&self) -> Result<Checkpoint> {
        let file = File::open("metadata_checkpoint.json").context("Failed to open checkpoint file")?;
        let reader = BufReader::new(file);
        let checkpoint: Checkpoint = serde_json::from_reader(reader).context("Failed to parse checkpoint file")?;
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

impl Default for Checkpoint {
    fn default() -> Self {
        Checkpoint {
            processed_article_ids: Vec::new(),
            processed_collection_ids: Vec::new(),
        }
    }
}

pub async fn run_metadata_generation() -> Result<()> {
    info!("Starting metadata generation process");
    let generator = MetadataGenerator::new(4).await.context("Failed to create MetadataGenerator")?;
    generator.generate_all_metadata().await.context("Failed to generate metadata")?;
    info!("Metadata generation process completed successfully");
    Ok(())
}