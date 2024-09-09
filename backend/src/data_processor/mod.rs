// File: src/data_processing/mod.rs

pub mod api_client;
pub mod convert_html;
pub mod generate_embedding;

pub use convert_html::html_to_markdown;
pub use generate_embedding::{generate_embeddings, store_embedding};
use log::info;

use crate::data_processor::api_client::ApiClient;
use crate::db::vector_db::init_vector_db;
use crate::db::DbPool;
use crate::job::Job;
use crate::models::{Article, ArticleRef, Collection};

use anyhow::Result;
use std::sync::Arc;

pub struct DataProcessor {
    pub api_client: ApiClient,
    db_pool: Arc<DbPool>,
    vector_db_client: Arc<qdrant_client::Qdrant>,
}

impl DataProcessor {
    pub async fn new(db_pool: Arc<DbPool>) -> Result<Self> {
        let api_client = ApiClient::new(None, None).map_err(|e| anyhow::anyhow!("{}", e))?;
        let vector_db_client = Arc::new(
            init_vector_db()
                .await
                .map_err(|e| anyhow::anyhow!("{}", e))?,
        );
        Ok(Self {
            api_client,
            db_pool,
            vector_db_client,
        })
    }

    pub async fn prepare_sync_collection(&self, collection: &Collection) -> Result<(), anyhow::Error> {
        info!("Preparing to sync collection: ID:{:?}, Slug: {:?}", collection.id, collection.slug);
        println!("Preparing to sync collection: ID:{:?}, Slug: {:?}", collection.id, collection.slug);

        // let mut jobs = vec![Job::StoreCollection(collection.clone())];
        self.sync_collection(collection).await?;

        let article_refs = self.api_client.get_list_articles(collection).await?;
        for article_ref in article_refs {
            // jobs.push(Job::SyncArticle(article_ref, collection.clone()));
            self.sync_article(&article_ref, &collection).await?;
        }

        Ok(())
        // Ok(jobs)
    }

    pub async fn sync_collection(&self, collection: &Collection) -> Result<()> {
        info!("Storing collection: ID:{:?}, Slug: {:?}", collection.id, collection.slug);
        println!("Storing collection: ID:{:?}, Slug: {:?}", collection.id, collection.slug);
        collection.store(&mut self.db_pool.get().expect("Failed to get DB connection"))?;
        Ok(())
    }

    pub async fn sync_article(
        &self,
        article_ref: &ArticleRef,
        collection: &Collection,
    ) -> Result<()> {
        let article = self
            .api_client
            .get_article(&article_ref.id.to_string(), collection)
            .await?;

        info!(
            "Processing article: ID:{:?}, Title: {:?}, Collection ID: {:?}, Helpscout Collection ID: {:?}", 
            article.id, 
            article.title, 
            collection.id, 
            collection.helpscout_collection_id
        );
        println!("Processing article: ID:{:?}, Title: {:?}", article.id, article.title);

        // Enqueue jobs for processing

        // let jobs = vec![
        //     Job::StoreArticle(article.clone()),
        //     Job::ConvertHtmlToMarkdown(article.clone()),
        //     Job::GenerateEmbeddings(article.clone()),
        // ];
        // Directly call the functions
        self.store_article(&article).await?;
        self.convert_html_to_markdown(&article).await?;
        self.generate_article_embeddings(&article).await?;

        // Ok(jobs)
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
        println!("Storing article: ID:{:?}, Title: {:?}", article.id, article.title);
        let article =article.store(&mut self.db_pool.get().expect("Failed to get DB connection"))?;
        Ok(article)
    }

    pub async fn convert_html_to_markdown(&self, article: &Article) -> Result<()> {
        info!("Converting HTML to Markdown for article: ID:{:?}, Title: {:?}", article.id, article.title);
        println!("Converting HTML to Markdown for article: ID:{:?}, Title: {:?}", article.id, article.title);
        let markdown = html_to_markdown(
            article
                .html_content
                .as_ref()
                .ok_or(anyhow::anyhow!("HTML content not found"))?,
        );
        article.update_markdown_content(
            &mut self.db_pool.get().expect("Failed to get DB connection"),
            markdown,
        )?;
        Ok(())
    }

    pub async fn generate_article_embeddings(&self, article: &Article) -> Result<()> {
        info!("Generating embeddings for article: ID:{:?}, Title: {:?}", article.id, article.title);
        println!("Generating embeddings for article: ID:{:?}, Title: {:?}", article.id, article.title);

        let embedding_and_point = generate_embeddings(article.clone())
            .await
            .expect("Failed to generate embeddings");

        store_embedding(
            embedding_and_point,
            self.vector_db_client.clone(),
            self.db_pool.clone(),
        )
        .await?;
        Ok(())
    }
}
