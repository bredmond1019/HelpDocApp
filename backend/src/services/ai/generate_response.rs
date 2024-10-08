use futures::stream::Stream;
use log::{error, info};
use ollama_rs::generation::{
    chat::{request::ChatMessageRequest, ChatMessage, ChatMessageResponseStream},
    completion::request::GenerationRequest,
};
use std::pin::Pin;
use tokio_stream::StreamExt;

use super::{AIModelError, AIService};

impl AIService {
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
        info!("Starting Model Generation");
        let model = "llama3.1:latest".to_string();

        let res = self
            .ollama
            .generate(GenerationRequest::new(model, input))
            .await;

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
}
