use futures::stream::Stream;
use log::{error, info};
use ollama_rs::{
    generation::{chat::{request::ChatMessageRequest, ChatMessage, ChatMessageResponseStream}, completion::request::GenerationRequest},
    Ollama,
};
use std::error::Error as StdError;
use std::fmt;
use std::pin::Pin;
use tokio_stream::StreamExt;

use crate::models::articles::Article;

#[derive(Clone)]
pub struct AIService {
    ollama: Ollama,
}

impl AIService {
    pub fn new() -> Self {
        info!("Initializing new AIService");
        Self {
            ollama: Ollama::new_default_with_history(30),
        }
    }

    pub async fn generate_stream_response(
        &mut self,
        input: String,
    ) -> Result<
        Pin<Box<dyn Stream<Item = Result<String, Box<dyn std::error::Error>>> + Send>>,
        Box<dyn std::error::Error + Send + Sync>,
    > {
        info!("Generating AI response for input: {}", input);
        let stream: ChatMessageResponseStream = self
            .ollama
            .send_chat_messages_with_history_stream(
                ChatMessageRequest::new(
                    "llama3.1:latest".to_string(),
                    vec![ChatMessage::user(input.clone())],
                ),
                "user".to_string(),
            )
            .await
            .map_err(|e| {
                error!("Failed to send chat message: {}", e);
                AIModelError::RequestError(e.to_string())
            })?;

        info!("Successfully initiated chat message stream");

        Ok(Box::pin(stream.map(|res| match res {
            Ok(chunk) => {
                if let Some(assistant_message) = chunk.message {
                    info!("Received chunk of AI response");
                    Ok(assistant_message.content)
                } else {
                    Ok(String::new())
                }
            }
            Err(e) => {
                error!("Error while streaming response: {:?}", e);
                Err(Box::new(AIModelError::StreamingError(format!("{:?}", e)))
                    as Box<dyn std::error::Error>)
            }
        })))
    }

    pub async fn generate_response(
        &self,
        input: String,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        info!("Generating AI response for input: {}", input);
        let model = "llama3.1:latest".to_string();

        let res = self.ollama.generate(GenerationRequest::new(model, input)).await;

        match res {
            Ok(response) => {
                info!("AI response generated successfully");
                Ok(response.response)
            },
            Err(e) => {
                error!("Failed to generate AI response: {}", e);
                Err(Box::new(AIModelError::RequestError(e.to_string())) as Box<dyn std::error::Error + Send + Sync>)
            }
        }
    }

    pub async fn generate_article_metadata(&self, article: &Article) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let prompt = format!(
            "Read the following article and provide:\n
            1. A paragraph description of the article\n
            2. 5-10 bullet points of important facts\n
            3. 5-20 keywords or phrases about the article\n

            You response should be in the following format:\n
            1. Response to part 1\n
            2. Response to part 2\n
            3. Response to part 3\n\n
            Article content:\n{}",
            article.markdown_content.as_deref().unwrap_or(&article.title)
        );

        let response = self.generate_response(prompt).await?;
        // let (paragraph, bullets, keywords) = self.parse_article_metadata(&response);

        Ok(response)
    }

    pub async fn generate_collection_metadata(&self, articles: &Vec<Article>) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
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
            You response should be in the following format:\n
            1. Response to part 1\n
            2. Response to part 2\n
            3. Response to part 3\n\n
            Collection metadata:\n{}",
            metadata
        );

        let response = self.generate_response(prompt).await?;
        // let (paragraph, bullets, keywords) = self.parse_article_metadata(&response);

        Ok(response)
    }
    
}

#[derive(Debug)]
enum AIModelError {
    RequestError(String),
    StreamingError(String)
}

impl fmt::Display for AIModelError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AIModelError::RequestError(e) => write!(f, "Request error: {}", e),
            AIModelError::StreamingError(e) => write!(f, "Streaming error: {}", e),
        }
    }
}

impl StdError for AIModelError {}
