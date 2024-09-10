use diesel::PgConnection;
use reqwest::Client;
use serde_json::json;
use uuid::Uuid;
use crate::models::embedding::Embedding;

async fn generate_and_store_embedding(
    conn: &mut PgConnection,
    article_id: Uuid,
    text: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Create a new reqwest client
    let client = Client::new();

    // Send a POST request to the Python embedding service
    let resp = client.post("http://localhost:5000/embed")
        .json(&json!({
            "text": text
        }))
        .send()
        .await?;

    // Parse the response
    let embedding_data: serde_json::Value = resp.json().await?;
    let embedding_vector: Vec<f32> = serde_json::from_value(embedding_data["embedding"].clone())?;

    // Create a new Embedding instance
    let embedding = Embedding::new(article_id, embedding_vector);

    // Store the embedding in the database
    embedding.store(conn)?;

    Ok(())
}