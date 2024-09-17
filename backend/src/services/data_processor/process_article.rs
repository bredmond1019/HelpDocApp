use actix_web::Result;
use diesel::PgConnection;
use log::{error, info, warn};
use pgvector::Vector;
use regex::Regex;

use super::{DataProcessor, ProcessResult};

use crate::models::Article;

impl DataProcessor {
    pub async fn process_article_metadata(
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
                    info!("Paragraph: {}", paragraph);
                    info!("Bullets: {:?}", bullets);
                    info!("Keywords: {:?}", keywords);

                    let mut result = ProcessResult::new(article.id);

                    if paragraph != "No summary available" {
                        result.paragraph = Some(paragraph.clone());
                    }
                    if bullets != vec!["No facts available"] {
                        result.bullets = Some(bullets.clone());
                    }
                    if keywords != vec!["No keywords available"] {
                        result.keywords = Some(keywords.clone());
                    }

                    if result.is_complete() {
                        let embeddings = self
                            .generate_embeddings(&paragraph, &bullets, &keywords)
                            .await?;
                        self.update_article_metadata(
                            &mut conn, article, paragraph, bullets, keywords, embeddings,
                        )?;
                        return Ok(result);
                    } else {
                        return Ok(result);
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

    fn parse_llm_response(
        &self,
        response: &str,
    ) -> Result<(String, Vec<String>, Vec<String>), Box<dyn std::error::Error + Send + Sync>> {
        let section_re = Regex::new(r"(?m)^\[([^\]]+)\]")?;
        let sections: Vec<_> = section_re.find_iter(response).collect();

        let mut summary = String::with_capacity(300);
        let mut facts = Vec::with_capacity(10);
        let mut keywords = Vec::with_capacity(20);

        for (i, section_match) in sections.iter().enumerate() {
            let section_name = &response[section_match.start() + 1..section_match.end() - 1];
            let content_start = section_match.end();
            let content_end = if i < sections.len() - 1 {
                sections[i + 1].start()
            } else {
                response.len()
            };
            let content = response[content_start..content_end].trim();

            match section_name {
                "SUMMARY" => summary = content.to_string(),
                "FACTS" => {
                    facts = content
                        .lines()
                        .map(|s| s.trim_start_matches('-').trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect()
                }
                "KEYWORDS" => {
                    keywords = content
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect()
                }
                _ => {}
            }
        }

        // Handle cases where a section is missing or empty
        if summary.is_empty() || summary == "N/A" {
            summary = "No summary available".to_string();
        }
        if facts.is_empty() {
            facts.push("No facts available".to_string());
        }
        if keywords.is_empty() {
            keywords.push("No keywords available".to_string());
        }

        Ok((summary, facts, keywords))
    }

    async fn generate_embeddings(
        &self,
        paragraph: &str,
        bullets: &[String],
        keywords: &[String],
    ) -> Result<(Vec<f32>, Vec<f32>, Vec<f32>), Box<dyn std::error::Error + Send + Sync>> {
        let joined_bullets = ProcessResult::join_with_commas(bullets.to_vec());
        let joined_keywords = ProcessResult::join_with_commas(keywords.to_vec());
        let paragraph_embedding = self.embedding_service.generate_embedding(paragraph).await?;

        let bullets_embedding = self
            .embedding_service
            .generate_embedding(&joined_bullets)
            .await?;
        let keywords_embedding = self
            .embedding_service
            .generate_embedding(&joined_keywords)
            .await?;

        info!("Paragraph embedding: {:?}", paragraph_embedding);
        info!("Bullets embedding: {:?}", bullets_embedding);
        info!("Keywords embedding: {:?}", keywords_embedding);

        Ok((paragraph_embedding, bullets_embedding, keywords_embedding))
    }

    fn update_article_metadata(
        &self,
        conn: &mut PgConnection,
        article: &Article,
        paragraph: String,
        bullets: Vec<String>,
        keywords: Vec<String>,
        embeddings: (Vec<f32>, Vec<f32>, Vec<f32>),
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let (paragraph_embedding, bullets_embedding, keywords_embedding) = embeddings;
        article.update_metadata(
            conn,
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
