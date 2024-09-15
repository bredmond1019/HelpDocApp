use super::DataProcessor;
use log::{info, error};
use crate::{data_processor::html_to_markdown, models::{Article, ArticleRef, Collection}};

use anyhow::{Context, Result};

impl DataProcessor {
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
}