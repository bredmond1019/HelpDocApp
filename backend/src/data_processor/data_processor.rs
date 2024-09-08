use crate::data_processor::{api_client::ApiClient, html_to_markdown};
use crate::db::vector_db::init_vector_db;
use crate::db::DbPool;
use crate::jobs::Job;
use crate::models::{Article, ArticleRef, Collection};

use anyhow::Result;
use std::sync::Arc;

use super::generate_embedding::{generate_embeddings, store_embedding};

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

    pub async fn prepare_sync_collection(&self, collection: &Collection) -> Result<Vec<Job>> {
        let mut jobs = vec![Job::StoreCollection(collection.clone())];

        let article_refs = self.api_client.get_list_articles(collection).await?;
        for article_ref in article_refs {
            jobs.push(Job::SyncArticle(article_ref, collection.clone()));
        }

        Ok(jobs)
    }

    pub async fn sync_collection(&self, collection: &Collection) -> Result<()> {
        collection.store(&mut self.db_pool.get().expect("Failed to get DB connection"))?;
        Ok(())
    }

    pub async fn sync_article(
        &self,
        article_ref: &ArticleRef,
        collection: &Collection,
    ) -> Result<Vec<Job>> {
        let article = self
            .api_client
            .get_article(&article_ref.id.to_string(), collection)
            .await?;

        let jobs = vec![
            Job::StoreArticle(article.clone()),
            Job::ConvertHtmlToMarkdown(article.clone()),
            Job::GenerateEmbeddings(article.clone()),
        ];

        Ok(jobs)
    }

    pub async fn store_article(&self, article: &Article) -> Result<()> {
        article.store(&mut self.db_pool.get().expect("Failed to get DB connection"))?;
        Ok(())
    }

    pub async fn convert_html_to_markdown(&self, article: &Article) -> Result<()> {
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
