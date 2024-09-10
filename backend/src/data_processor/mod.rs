// File: src/data_processing/mod.rs

pub mod api_client;
pub mod convert_html;

pub use convert_html::html_to_markdown;
use log::{info, error};

use crate::data_processor::api_client::ApiClient;
use crate::db::DbPool;
use crate::models::{Article, ArticleRef, Collection};

use anyhow::{Context, Result};
use std::sync::Arc;

pub struct DataProcessor {
    pub api_client: ApiClient,
    db_pool: Arc<DbPool>,
}

impl DataProcessor {
    pub async fn new(db_pool: Arc<DbPool>) -> Result<Self> {
        let api_client = ApiClient::new(None, None).map_err(|e| anyhow::anyhow!("{}", e))?;
        
        info!("DataProcessor initialization complete");
        Ok(Self {
            api_client,
            db_pool,
        })
    }

    pub async fn prepare_sync_collection(&self, collection: &Collection) -> Result<(), anyhow::Error> {
        info!("Preparing to sync collection: ID:{:?}, Slug: {:?}", collection.id, collection.slug);
        self.sync_collection(collection).await?;

        let article_refs = self.api_client.get_list_articles(collection).await?;
        for article_ref in article_refs {
            self.sync_article(&article_ref, &collection).await?;
        }

        Ok(())
    }

    pub async fn sync_collection(&self, collection: &Collection) -> Result<()> {
        info!("Storing collection: ID:{:?}, Slug: {:?}", collection.id, collection.slug);
        let mut conn = self.db_pool.get()
            .context("Failed to get DB connection")?;

        collection.store(&mut conn)
            .with_context(|| format!("Failed to store collection: ID:{:?}, Slug:{:?}", collection.id, collection.slug))?;

        info!("Successfully stored collection: ID:{:?}, Slug:{:?}", collection.id, collection.slug);
        Ok(())
    }

    pub async fn sync_article(
        &self,
        article_ref: &ArticleRef,
        collection: &Collection,
    ) -> Result<()> {
        let article = match self.api_client.get_article(&article_ref.id.to_string(), collection).await {
            Ok(article) => article,
            Err(e) => {
                error!("Failed to fetch article ID:{}: {}", article_ref.id, e);
                return Err(anyhow::anyhow!("Failed to fetch article: {}", e));
            }
        };

        info!(
            "Processing article: ID:{:?}, Title: {:?}, Collection ID: {:?}, Helpscout Collection ID: {:?}", 
            article.id, 
            article.title, 
            collection.id, 
            collection.helpscout_collection_id
        );

        if let Err(e) = self.store_article(&article).await {
            error!("Failed to store article ID:{}: {}", article.id, e);
            return Err(anyhow::anyhow!("Failed to store article: {}", e));
        }

        if let Err(e) = self.convert_html_to_markdown(&article).await {
            error!("Failed to convert HTML to Markdown for article ID:{}: {}", article.id, e);
            return Err(anyhow::anyhow!("Failed to convert HTML to Markdown: {}", e));
        }

        // if let Err(e) = self.generate_article_embeddings(&article).await {
        //     error!("Failed to generate embeddings for article ID:{}: {}", article.id, e);
        //     return Err(anyhow::anyhow!("Failed to generate article embeddings: {}", e));
        // }

        Ok(())
    }

    pub async fn store_article(&self, article: &Article) -> Result<Article> {
        info!(
            "Storing article: ID:{:?}, Title: {:?}, Collection ID: {:?}, Helpscout Collection ID: {:?}",
            article.id,
            article.title,
            article.collection_id,
            article.helpscout_collection_id
        );
        let article =article.store(&mut self.db_pool.get().expect("Failed to get DB connection"))?;
        Ok(article)
    }

    pub async fn convert_html_to_markdown(&self, article: &Article) -> Result<()> {
        info!("Converting HTML to Markdown for article: ID:{:?}, Title: {:?}", article.id, article.title);
        let markdown = html_to_markdown(
            article
                .html_content
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("HTML content not found for article ID:{}", article.id))?
        )?;

        article.update_markdown_content(
            &mut *self.db_pool.get().context("Failed to get DB connection")?,
            markdown,
        ).context(format!("Failed to update markdown content for article ID:{}", article.id))?;
        
        Ok(())
    }

    // pub async fn generate_article_embeddings(&self, article: &Article) -> Result<()> {
    //     info!("Generating embeddings for article: ID:{:?}, Title: {:?}", article.id, article.title);

    //     let embedding_and_point = generate_embeddings(article.clone())
    //         .await
    //         .map_err(|e| anyhow::anyhow!("{}", e))
    //         .context(format!("Failed to generate embeddings for article ID:{}", article.id))?;

    //     store_embedding(
    //         embedding_and_point,
    //         self.vector_db_client.clone(),
    //         self.db_pool.clone(),
    //     )
    //     .await
    //     .context(format!("Failed to store embedding for article ID:{}", article.id))?;
        
    //     Ok(())
    // }
}
