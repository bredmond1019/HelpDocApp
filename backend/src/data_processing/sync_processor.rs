use crate::data_processing::{
    fetcher::ApiClient, generate_embeddings, html_to_markdown, store_in_postgres,
};
use crate::db::vector_db::init_vector_db;
use crate::db::DbPool;
use crate::models::{Article, Collection};
use anyhow::Result;
use std::sync::Arc;

pub struct SyncProcessor {
    api_client: ApiClient,
    db_pool: Arc<DbPool>,
    vector_db: Arc<qdrant_client::Qdrant>,
}

impl SyncProcessor {
    pub async fn new(db_pool: Arc<DbPool>) -> Result<Self> {
        let api_client = ApiClient::new().map_err(|e| anyhow::anyhow!("{}", e))?;
        let vector_db = Arc::new(
            init_vector_db()
                .await
                .map_err(|e| anyhow::anyhow!("{}", e))?,
        );
        Ok(Self {
            api_client,
            db_pool,
            vector_db,
        })
    }

    pub async fn sync_all(&self) -> Result<()> {
        let collections = self.api_client.list_collections(None).await?;

        for collection in collections {
            self.process_collection(&collection).await?;
        }

        Ok(())
    }

    async fn process_collection(&self, collection: &Collection) -> Result<()> {
        println!("Processing collection: {}", collection.name);

        // Store the collection in the database
        store_in_postgres(&self.db_pool, collection, &[]).await?;

        let articles = self
            .api_client
            .list_articles(&collection.id.to_string(), None)
            .await?;
        let mut processed_articles = Vec::new();

        for article_ref in articles {
            let full_article = self
                .api_client
                .get_article(&article_ref.id.to_string())
                .await?;
            let processed_article = self.process_article(full_article).await?;
            processed_articles.push(processed_article);
        }

        // Store all articles for this collection
        store_in_postgres(&self.db_pool, collection, &processed_articles).await?;

        // Generate and store embeddings
        let embeddings = generate_embeddings(&self.vector_db, &processed_articles)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to generate embeddings: {}", e))?;

        // TODO: Store embeddings in the database if needed

        Ok(())
    }

    async fn process_article(&self, mut article: Article) -> Result<Article> {
        println!("Processing article: {}", article.title);

        // Convert HTML content to Markdown
        if let Some(html_content) = &article.html_content {
            article.markdown_content = Some(html_to_markdown(html_content));
        }

        Ok(article)
    }
}
