use actix_web::Result;
use log::{error, info};
use pgvector::Vector;

use super::DataProcessor;

use crate::models::Article;

impl DataProcessor {
    pub async fn process_article_metadata(
        &self,
        article: &Article,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut conn = self.db_pool.get()?;
        let response = self.ai_service.generate_article_metadata(article).await?;
        let (paragraph, bullets, keywords) = self.parse_metadata(&response);

        info!("Response for article: {}: {}", article.id, response);
        info!("Paragraph: {}", paragraph);
        info!("Bullets: {:?}", bullets);
        info!("Keywords: {:?}", keywords);

        if paragraph.is_empty() || bullets.is_empty() || keywords.is_empty() {
            error!(
                "Metadata generation failed for article: {},\n title:{},\n response: {}",
                article.id, article.title, response
            );
        }

        let paragraph_embedding = self
            .embedding_service
            .generate_embedding(&paragraph)
            .await?;
        let bullets_embedding = self.embedding_service.generate_embedding(&bullets).await?;
        let keywords_embedding = self.embedding_service.generate_embedding(&keywords).await?;

        info!("Paragraph embedding: {:?}", paragraph_embedding);
        info!("Bullets embedding: {:?}", bullets_embedding);
        info!("Keywords embedding: {:?}", keywords_embedding);

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

    pub fn parse_metadata(&self, response: &str) -> (String, String, String) {
        let mut paragraph = String::new();
        let mut bullets = String::new();
        let mut keywords = String::new();
        let mut current_section = 0;

        for line in response.lines() {
            if line.trim().is_empty() {
                continue;
            }

            if line.starts_with("**1. Response to part 1**") {
                current_section = 1;
            } else if line.starts_with("**2. Response to part 2**") {
                current_section = 2;
            } else if line.starts_with("**3. Response to part 3**") {
                current_section = 3;
            } else {
                match current_section {
                    1 => paragraph.push_str(line.trim()),
                    2 => {
                        if line.starts_with('*') {
                            bullets.push_str(line.trim_start_matches('*').trim());
                            bullets.push('\n');
                        }
                    }
                    3 => {
                        if line.starts_with('*') {
                            keywords.push_str(line.trim_start_matches('*').trim());
                            keywords.push('\n');
                        } else if line.starts_with("Keywords: ") {
                            keywords.push_str(line.trim_start_matches("Keywords: ").trim());
                            keywords.push('\n');
                        } else {
                            keywords.push_str(line.trim());
                            keywords.push('\n');
                        }
                    }
                    _ => {}
                }
            }
        }

        (
            paragraph.trim().to_string(),
            bullets.trim().to_string(),
            keywords.trim().to_string(),
        )
    }
}
