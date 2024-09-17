use actix_web::Result;
use log::{error, info, warn};

use super::{DataProcessor, ProcessResult};
use crate::models::Article;

impl DataProcessor {
    pub async fn process_failed_article_metadata(
        &self,
        article: &Article,
    ) -> Result<ProcessResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut conn = self.db_pool.get()?;
        let mut attempts = 0;
        const MAX_ATTEMPTS: u8 = 3;

        while attempts < MAX_ATTEMPTS {
            let response = self.ai_service.generate_article_metadata(article).await?;
            match self.parse_llm_response(&response) {
                Ok((paragraph, bullets, keywords)) => {
                    info!("Response for article: {}: {}", article.id, response);
                    let mut result = ProcessResult::new(article.id);

                    // Paragraph description
                    result.paragraph = article.paragraph_description.clone();
                    result.paragraph_description_embedding =
                        article.paragraph_description_embedding.clone();
                    if result.paragraph.is_none() && paragraph != "No summary available" {
                        result.paragraph = Some(paragraph.clone());
                        result.paragraph_description_embedding =
                            Some(self.generate_embedding(&paragraph).await?);
                    }

                    // Bullet points
                    result.bullets = article.bullet_points.clone();
                    result.bullet_points_embedding = article.bullet_points_embedding.clone();
                    if result.bullets.is_none() && bullets != vec!["No facts available"] {
                        result.bullets = Some(bullets.clone().into_iter().map(Some).collect());
                        result.bullet_points_embedding =
                            Some(self.generate_embedding(&bullets.join(", ")).await?);
                    }

                    // Keywords
                    result.keywords = article.keywords.clone();
                    result.keywords_embedding = article.keywords_embedding.clone();
                    if result.keywords.is_none() && keywords != vec!["No keywords available"] {
                        result.keywords = Some(keywords.clone().into_iter().map(Some).collect());
                        result.keywords_embedding =
                            Some(self.generate_embedding(&keywords.join(", ")).await?);
                    }

                    if result.is_complete() {
                        return Ok(result);
                    } else if result.has_content() {
                        info!("Updating metadata for article: {}", article.id);
                        info!("Metadata: {:?}", result.clone());

                        match article.update_metadata(&mut conn, result.clone()) {
                            Ok(_) => return Ok(result),
                            Err(e) => {
                                warn!("Failed to update metadata in database for article: {}. Error: {}. Saving to file instead.", article.id, e);
                                self.save_metadata_to_file(article.id, &result)?;
                                return Ok(result);
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("Error parsing response: {}", e);
                }
            }

            attempts += 1;
            if attempts < MAX_ATTEMPTS {
                warn!("Retrying metadata generation for article: {}", article.id);
            }
        }

        error!(
            "Failed to generate valid metadata for article: {} after {} attempts",
            article.id, MAX_ATTEMPTS
        );
        Ok(ProcessResult::new(article.id))
    }
}
