use log::{error, info};
use ollama_rs::generation::completion::request::GenerationRequest;
use std::sync::Arc;

use crate::{models::Article, services::ai::AIModelError};

use super::ollama_load_balancer::OllamaLoadBalancer;

pub struct AIDataService {
    ollama_balancer: Arc<OllamaLoadBalancer>,
}

impl AIDataService {
    pub fn new(ollama_balancer: Arc<OllamaLoadBalancer>) -> Self {
        Self { ollama_balancer }
    }

    pub async fn generate_response(
        &self,
        input: String,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let ollama = self.ollama_balancer.get_server().await;
        let ollama = ollama.lock().await;

        info!("Generating AI response for current prompt");
        let model = "llama3.1:latest".to_string();

        let res = ollama.generate(GenerationRequest::new(model, input)).await;

        match res {
            Ok(response) => {
                info!("AI response generated successfully");
                Ok(response.response)
            }
            Err(e) => {
                error!("Failed to generate AI response: {}", e);
                Err(Box::new(AIModelError::RequestError(e.to_string()))
                    as Box<dyn std::error::Error + Send + Sync>)
            }
        }
    }

    pub async fn generate_article_metadata(
        &self,
        article: &Article,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let prompt = format!(
            r#"Analyze the following article and provide a structured response EXACTLY as specified below. Follow these instructions precisely:
        
        1. Your response MUST contain these three sections in this order: [SUMMARY], [FACTS], and [KEYWORDS].
        2. Each section MUST be preceded by its header in square brackets.
        3. Do not include any text before [SUMMARY] or after [KEYWORDS].
        4. If you cannot provide content for a section, use "N/A" as the content.
        
        [SUMMARY]
        Provide a concise one-paragraph summary of the article's main points. If unable to summarize, write "N/A".
        
        [FACTS]
        List 5-10 important facts from the article, each on a new line starting with a dash (-). If unable to extract facts, write "N/A".
        
        [KEYWORDS]
        List relevant keywords or phrases, separated by commas, to improve search results. Use 1-2 words per term. If unable to provide keywords, write "N/A".
        
        Article content:
        {}"#,
            article
                .markdown_content
                .as_deref()
                .unwrap_or(&article.title)
        );

        info!("Generating AI response for current prompt");

        let response = self.generate_response(prompt).await?;

        Ok(response)
    }
}
