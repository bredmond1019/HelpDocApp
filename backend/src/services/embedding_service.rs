use diesel::PgConnection;
use reqwest::Client;
use serde_json::json;
use uuid::Uuid;
use log::{info, error};
use anyhow::{Result, Context};

use crate::models::embedding::Embedding;

pub async fn generate_embedding(text: &str) -> Result<Vec<f32>> {
    let client = Client::new();

    let resp = client.post("http://localhost:8080/embed")
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
    conn: &mut PgConnection,
    article_id: Uuid,
    text: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Generating embedding for article {}", article_id);

    let embedding_vector = generate_embedding(text).await?;
    store_embedding(conn, article_id, embedding_vector)?;

    info!("Successfully generated and stored embedding for article {}", article_id);
    Ok(())
}

fn store_embedding(conn: &mut PgConnection, article_id: Uuid, embedding_vector: Vec<f32>) -> Result<(), Box<dyn std::error::Error>> {
    let embedding = Embedding::new(article_id, embedding_vector);
    embedding.store(conn).map_err(|e| {
        error!("Failed to store embedding for article {}: {}", article_id, e);
        e
    })?;
    Ok(())
}




