use log::{error, info};
use ollama_rs::generation::completion::request::GenerationRequest;
use std::sync::Arc;

use crate::services::ai::AIModelError;

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

    // Other methods...
}
