// This could be in a separate file, e.g., embedding_service.rs

use diesel::PgConnection;
use reqwest::Client;
use serde_json::json;
use uuid::Uuid;
use crate::models::embedding::Embedding;
use log::{info, error};

pub async fn generate_and_store_embedding(
    conn: &mut PgConnection,
    article_id: Uuid,
    text: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Generating embedding for article {}", article_id);

    // Create a new reqwest client
    let client = Client::new();

    // Send a POST request to the Python embedding service
    let resp = client.post("http://localhost:5000/embed")
        .json(&json!({
            "text": text
        }))
        .send()
        .await
        .map_err(|e| {
            error!("Failed to send request to embedding service: {}", e);
            e
        })?;

    // Check if the response is successful
    if !resp.status().is_success() {
        let error_message = resp.text().await?;
        error!("Embedding service returned an error: {}", error_message);
        return Err(format!("Embedding service error: {}", error_message).into());
    }

    // Parse the response
    let embedding_data: serde_json::Value = resp.json().await.map_err(|e| {
        error!("Failed to parse embedding service response: {}", e);
        e
    })?;

    let embedding_vector: Vec<f32> = serde_json::from_value(embedding_data["embedding"].clone())
        .map_err(|e| {
            error!("Failed to extract embedding vector from response: {}", e);
            e
        })?;

    // Create a new Embedding instance
    let embedding = Embedding::new(article_id, embedding_vector);

    // Store the embedding in the database
    embedding.store(conn).map_err(|e| {
        error!("Failed to store embedding for article {}: {}", article_id, e);
        e
    })?;

    info!("Successfully generated and stored embedding for article {}", article_id);
    Ok(())
}