use actix_web::Result;
use log::{error, info, warn};
use regex::Regex;

use super::DataProcessor;

use crate::models::Article;

impl DataProcessor {
    pub async fn test_process_metadata(
        &self,
        article: &Article,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut conn = self.db_pool.get()?;
        let mut attempts = 0;
        const MAX_ATTEMPTS: u8 = 3;

        while attempts < MAX_ATTEMPTS {
            let response = self
                .ai_data_service
                .generate_article_metadata(article)
                .await?;
            match self.parse_llm_response(&response) {
                Ok((paragraph, bullets, keywords)) => {
                    info!("Response for article: {}: {}", article.id, response);
                    info!("Paragraph: {}", paragraph);
                    info!("Bullets: {:?}", bullets);
                    info!("Keywords: {:?}", keywords);

                    if paragraph != "No summary available"
                        && bullets != vec!["No facts available"]
                        && keywords != vec!["No keywords available"]
                    {
                        // Successfully parsed and extracted meaningful content
                        return Ok(());
                    }
                }
                Err(e) => {
                    error!("Error parsing response: {}", e);
                }
            }

            attempts += 1;
            if attempts < MAX_ATTEMPTS {
                warn!("Retrying metadata generation for article: {}", article.id);
                // TODO: Implement exponential backoff?
                // TODO: Think about how to handle the case where the LLM returns a valid response
                // but the parsing fails. Should we retry the LLM call or move on?
            }
        }

        error!(
            "Failed to generate valid metadata for article: {} after {} attempts",
            article.id, MAX_ATTEMPTS
        );
        Ok(())
    }

    fn parse_llm_response(
        &self,
        response: &str,
    ) -> Result<(String, Vec<String>, Vec<String>), Box<dyn std::error::Error>> {
        let section_re = Regex::new(r"(?m)^\[([^\]]+)\]")?;
        let sections: Vec<_> = section_re.find_iter(response).collect();

        let mut summary = String::new();
        let mut facts = Vec::new();
        let mut keywords = Vec::new();

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
}
