// File: src/db/vector_db.rs

use qdrant_client::config::QdrantConfig;
use qdrant_client::qdrant::{
    vectors_config::Config, CreateCollection, Distance, VectorParams, VectorsConfig,
};
use qdrant_client::Qdrant;
use std::env;

pub async fn init_vector_db() -> Result<Qdrant, Box<dyn std::error::Error>> {
    let qdrant_url = env::var("QDRANT_URL").expect("QDRANT_URL not set");
    let config = QdrantConfig::from_url(&qdrant_url);
    let client = Qdrant::new(config)?;

    // Check if the collection already exists
    let collections = client.list_collections().await?;
    if !collections
        .collections
        .iter()
        .any(|c| c.name == "article_embeddings")
    {
        // Create a new collection for article embeddings
        let create_collection = CreateCollection {
            collection_name: "article_embeddings".to_string(),
            vectors_config: Some(VectorsConfig {
                config: Some(Config::Params(VectorParams {
                    size: 384, // Adjust this based on your embedding model
                    distance: Distance::Cosine.into(),
                    ..Default::default()
                })),
            }),
            ..Default::default()
        };

        client.create_collection(create_collection).await?;
        println!("Created 'article_embeddings' collection");
    } else {
        println!("'article_embeddings' collection already exists");
    }

    Ok(client)
}

pub async fn init_test_vector_db() -> Result<Qdrant, Box<dyn std::error::Error>> {
    let qdrant_url = env::var("QDRANT_URL").unwrap_or_else(|_| "http://localhost:6334".to_string());
    let config = QdrantConfig::from_url(&qdrant_url);
    let client = Qdrant::new(config)?;

    // Check if the collection already exists
    let collections = client.list_collections().await?;
    if !collections.collections.iter().any(|c| c.name == "testing") {
        // Create a new collection for article embeddings
        let create_collection = CreateCollection {
            collection_name: "testing".to_string(),
            vectors_config: Some(VectorsConfig {
                config: Some(Config::Params(VectorParams {
                    size: 384, // Adjust this based on your embedding model
                    distance: Distance::Cosine.into(),
                    ..Default::default()
                })),
            }),
            ..Default::default()
        };

        client.create_collection(create_collection).await?;
        println!("Created 'testing' collection");
    } else {
        println!("'testing' collection already exists");
    }

    Ok(client)
}
