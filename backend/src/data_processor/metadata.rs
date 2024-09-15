use actix_web::Result;
use pgvector::Vector;

use super::DataProcessor;

use crate::models::{Article, Collection};

impl DataProcessor {
    pub async fn generate_article_metadata(&self, article: &Article) -> Result<(String, String, String), Box<dyn std::error::Error + Send + Sync>> {
        let prompt = format!(
            "Read the following article and provide:\n
            1. A paragraph description of the article\n
            2. 5-10 bullet points of important facts\n
            3. 5-20 keywords or phrases about the article\n\n
            Article content:\n{}",
            article.markdown_content.as_deref().unwrap_or(&article.title)
        );

        let response = self.ai_model.generate_response(prompt).await?;
        let (paragraph, bullets, keywords) = self.parse_article_metadata(&response);

        Ok((paragraph, bullets, keywords))
    }

    fn parse_article_metadata(&self, response: &str) -> (String, String, String) {
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

    pub async fn update_article_metadata(&self, article: &Article) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let (paragraph, bullets, keywords) = self.generate_article_metadata(article).await?;
        
        let paragraph_embedding = self.embedding_service.generate_embedding(&paragraph).await?;
        let bullets_embedding = self.embedding_service.generate_embedding(&bullets).await?;
        let keywords_embedding = self.embedding_service.generate_embedding(&keywords).await?;

        let mut conn = self.db_pool.get()?;
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

    pub async fn generate_collection_metadata(&self, collection: &Collection) -> Result<(String, String, String), Box<dyn std::error::Error + Send + Sync>> {
        let mut conn = self.db_pool.get()?;
        let articles = Article::belonging_to(collection).load::<Article>(&mut conn)?;

        let metadata = articles.iter()
            .map(|article| {
                format!(
                    "Article: {}\n
                    Description: {}\n
                    Bullet Points: {}\n
                    Keywords: {}\n",
                    article.title,
                    article.paragraph_description.as_deref().unwrap_or(""),
                    article.bullet_points.as_deref().unwrap_or(""),
                    article.keywords.as_deref().unwrap_or("")
                )
            })
            .collect::<Vec<_>>()
            .join("\n\n");

        let prompt = format!(
            "Based on the following metadata from articles in a collection, provide:\n
            1. A paragraph description of the collection\n
            2. 5-10 bullet points summarizing the collection\n
            3. 5-20 keywords or phrases about the collection\n\n
            Collection metadata:\n{}",
            metadata
        );

        let response = self.ai_model.generate_response(prompt).await?;
        let (paragraph, bullets, keywords) = self.parse_article_metadata(&response);

        Ok((paragraph, bullets, keywords))
    }

    pub async fn update_collection_metadata(&self, collection: &Collection) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let (paragraph, bullets, keywords) = self.generate_collection_metadata(collection).await?;
        
        let paragraph_embedding = self.embedding_service.generate_embedding(&paragraph).await?;
        let bullets_embedding = self.embedding_service.generate_embedding(&bullets).await?;
        let keywords_embedding = self.embedding_service.generate_embedding(&keywords).await?;

        let mut conn = self.db_pool.get()?;
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