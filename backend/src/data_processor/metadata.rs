use actix_web::Result;
use pgvector::Vector;

use super::DataProcessor;

use crate::models::{Article, Collection};

impl DataProcessor {
    pub async fn process_article_metadata(&self, article: &Article) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut conn = self.db_pool.get()?;
        let response = self.ai_service.generate_article_metadata(article).await?;
        let (paragraph, bullets, keywords) = self.parse_metadata(&response);

        let paragraph_embedding = self.embedding_service.generate_embedding(&paragraph).await?;
        let bullets_embedding = self.embedding_service.generate_embedding(&bullets).await?;
        let keywords_embedding = self.embedding_service.generate_embedding(&keywords).await?;

        article.update_metadata(
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


    pub async fn process_collection_metadata(&self, collection: &Collection) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut conn = self.db_pool.get()?;
        let articles = Article::belonging_to_collection(collection, &mut conn)?;

        let response = self.ai_service.generate_collection_metadata(&articles).await?;
        let (paragraph, bullets, keywords) = self.parse_metadata(&response);

        let paragraph_embedding = self.embedding_service.generate_embedding(&paragraph).await?;
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

    fn parse_metadata(&self, response: &str) -> (String, String, String) {
        // Implement parsing logic here
        // This is a simplified version; you might need to adjust based on the actual AI output
        let mut paragraph = String::new();
        let mut bullets = String::new();
        let mut keywords = String::new();
        let mut current_section = 0;

        for line in response.lines() {
            if line.starts_with("1.") {
                current_section = 1;
            } else if line.starts_with("2.") {
                current_section = 2;
            } else if line.starts_with("3.") {
                current_section = 3;
            } else if line.starts_with("4.") {
                current_section = 4;
            } else {
                match current_section {
                    1 => paragraph.push_str(line),
                    2 => bullets.push_str(line),
                    3 => keywords.push_str(line),
                    _ => {}
                }
            }
        }

        (paragraph.trim().to_string(), bullets.trim().to_string(), keywords.trim().to_string())
    }
}