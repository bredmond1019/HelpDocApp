use actix_web::Result;
use pgvector::Vector;

use super::DataProcessor;
use crate::models::{Article, Collection};

impl DataProcessor {
    pub async fn process_collection_metadata(
        &self,
        collection: &Collection,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut conn = self.db_pool.get()?;
        let articles = Article::belonging_to_collection(collection, &mut conn)?;

        let response = self
            .ai_service
            .generate_collection_metadata(&articles)
            .await?;
        let (paragraph, bullets, keywords) = self.parse_metadata(&response);

        let paragraph_embedding = self
            .embedding_service
            .generate_embedding(&paragraph)
            .await?;
        let bullets_embedding = self.embedding_service.generate_embedding(&bullets).await?;
        let keywords_embedding = self.embedding_service.generate_embedding(&keywords).await?;

        collection.update_metadata(
            &mut conn,
            paragraph,
            bullets,
            keywords,
            Vector::from(paragraph_embedding),
            Vector::from(bullets_embedding),
            Vector::from(keywords_embedding),
        )?;

        Ok(())
    }
}
