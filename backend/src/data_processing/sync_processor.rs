use crate::data_processing::{fetcher::ApiClient, generate_embeddings, html_to_markdown};
use crate::db::vector_db::init_vector_db;
use crate::db::DbPool;
use crate::models::{Article, Collection};
use anyhow::Result;
use qdrant_client::qdrant::UpsertPointsBuilder;
use std::sync::Arc;

pub struct SyncProcessor {
    api_client: ApiClient,
    db_pool: Arc<DbPool>,
    vector_db_client: Arc<qdrant_client::Qdrant>,
}

impl SyncProcessor {
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

    pub async fn sync_all(&self) -> Result<()> {
        let collections = self.api_client.get_list_collections().await?;

        for collection in collections {
            self.process_collection(&collection).await?;
        }

        Ok(())
    }

    async fn process_collection(&self, collection: &Collection) -> Result<()> {
        println!("Processing collection: {}", collection.name);

        collection.store(&mut self.db_pool.get().expect("Failed to get DB connection"))?;

        let article_refs = self.api_client.get_list_articles(collection).await?;
        let mut processed_articles = Vec::new();

        for article_ref in article_refs {
            let article = self
                .api_client
                .get_article(&article_ref.id.to_string(), collection)
                .await?;
            let processed_article = self.process_article(article).await?;
            processed_articles.push(processed_article);
        }

        // Generate and store embeddings
        let embeddings_and_points = generate_embeddings(processed_articles.clone())
            .await
            .map_err(|e| anyhow::anyhow!("Failed to generate embeddings: {}", e))?;

        let (embeddings, points) = embeddings_and_points;

        self.vector_db_client
            .upsert_points(UpsertPointsBuilder::new("article_embeddings", points))
            .await?;

        embeddings.iter().for_each(|embedding| {
            embedding
                .store(&mut self.db_pool.get().expect("Failed to get DB connection"))
                .expect("Failed to store embedding");
        });

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
