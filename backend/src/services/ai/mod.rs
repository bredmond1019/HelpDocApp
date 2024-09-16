use log::info;
use ollama_rs::Ollama;
use std::error::Error as StdError;
use std::fmt;

pub mod generate_metadata;
pub mod generate_response;

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
}

#[derive(Debug)]
enum AIModelError {
    RequestError(String),
    StreamingError(String),
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
