use diesel::PgConnection;
use reqwest::Client;
use serde_json::json;
use uuid::Uuid;
use log::{info, error};
use anyhow::{Result, Context};

use crate::models::{article::ArticleChunk, embedding::Embedding, Article};
use crate::schema::article_chunks;

pub struct EmbeddingService {
    client: Client,
}

impl EmbeddingService {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }
    pub async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>> {
        let resp = self.client.post("http://localhost:8080/embed")
            .json(&json!({ "text": text }))
            .send()
            .await
            .map_err(|e| {
                error!("Failed to send request to embedding service: {}", e);
                e
            })?;

        if !resp.status().is_success() {
            let status = resp.status();
            let error_message = resp.text().await?;
            error!("Embedding service returned an error: {}", error_message);
            return Err(anyhow::anyhow!("Embedding service error: {} - {}", status, error_message));
        }

        let embedding_data: serde_json::Value = resp.json().await.map_err(|e| {
            error!("Failed to parse embedding service response: {}", e);
            e
        })?;

        serde_json::from_value(embedding_data["embedding"].clone())
            .context("Failed to extract embedding vector from response")
    }

    pub async fn generate_and_store_embedding(
        &self,
        conn: &mut PgConnection,
        article: &Article,
    ) -> Result<(), Box<dyn std::error::Error>> {
        info!("Generating embedding for article {}", article.id);

        let chunks = article.create_chunks(500); // 500 characters per chunk
        for mut chunk in chunks {
            let embedding_vector = self.generate_embedding(&chunk.content).await?;
            let embedding = Embedding::new(article.id, embedding_vector.clone());
            let stored_embedding = self.store_embedding(conn, embedding)?;

            chunk.embedding_id = Some(stored_embedding.id);
            chunk.store(conn)?;
        }

        info!("Successfully generated and stored embedding for article {}", article.id);
        Ok(())
    }

    fn store_embedding(
        &self,
        conn: &mut PgConnection,
        embedding: Embedding,
    ) -> Result<Embedding, Box<dyn std::error::Error>> {
        let stored_embedding = embedding.store(conn).map_err(|e| {
            error!("Failed to store embedding for article {}: {}", embedding.article_id, e);
            e
        })?;
        Ok(stored_embedding)
    }
}
