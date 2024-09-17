use actix_web::Result;
use diesel::PgConnection;
use log::{error, info, warn};
use pgvector::Vector;
use regex::Regex;
use std::fs::OpenOptions;
use std::io::Write;
use uuid::Uuid;

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
                        result.paragraph_description_embedding =
                            Some(self.generate_embedding(&paragraph).await?);
                    }
                    if bullets != vec!["No facts available"] {
                        result.bullets = Some(bullets.clone());
                        result.bullet_points_embedding =
                            Some(self.generate_embedding(&bullets.join(", ")).await?);
                    }
                    if keywords != vec!["No keywords available"] {
                        result.keywords = Some(keywords.clone());
                        result.keywords_embedding =
                            Some(self.generate_embedding(&keywords.join(", ")).await?);
                    }

                    if result.has_content() {
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
                    } else {
                        warn!("No content found in response for article: {}", article.id);
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

    async fn generate_embedding(
        &self,
        text: &str,
    ) -> Result<Vector, Box<dyn std::error::Error + Send + Sync>> {
        let embedding = self.embedding_service.generate_embedding(text).await?;
        Ok(Vector::from(embedding))
    }

    fn save_metadata_to_file(
        &self,
        article_id: Uuid,
        result: &ProcessResult,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let file_name = format!("failed_metadata_updates_{}.txt", article_id);
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .append(true)
            .open(file_name)?;

        writeln!(file, "Article ID: {}", article_id)?;
        writeln!(file, "Paragraph: {:?}", result.paragraph)?;
        writeln!(file, "Bullets: {:?}", result.bullets)?;
        writeln!(file, "Keywords: {:?}", result.keywords)?;
        writeln!(file, "---")?;

        Ok(())
    }
}
