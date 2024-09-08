use crate::data_processing::generate_embedding::generate_article_embeddings;
use crate::data_processing::{fetcher::ApiClient, html_to_markdown};
use crate::db::vector_db::init_vector_db;
use crate::db::DbPool;
use crate::models::article::ArticleRef;
use crate::models::{Article, Collection};
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

    pub async fn sync_collection(&self, collection: &Collection) -> Result<()> {
        println!("Processing collection: {}", collection.name);

        collection.store(&mut self.db_pool.get().expect("Failed to get DB connection"))?;

        let article_refs = self.api_client.get_list_articles(collection).await?;

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
        let processed_article = self.process_article(article).await?;

        // Generate and store embeddings
        generate_article_embeddings(
            vec![processed_article],
            self.vector_db_client.clone(),
            self.db_pool.clone(),
        )
        .await?;

        Ok(())
    }

    async fn process_article(&self, mut article: Article) -> Result<Article> {
        println!("Processing article: {}", article.title);

        // Convert HTML content to Markdown
        if let Some(html_content) = &article.html_content {
            article.markdown_content = Some(html_to_markdown(html_content));
        }

        article.store(&mut self.db_pool.get().expect("Failed to get DB connection"))?;

        Ok(article)
    }
}
