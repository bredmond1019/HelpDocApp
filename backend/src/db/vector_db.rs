// File: src/db/vector_db.rs

use qdrant_client::config::QdrantConfig;

use qdrant_client::qdrant::{CreateCollection, Distance, VectorParams, VectorsConfig};
use qdrant_client::Qdrant;

pub async fn init_vector_db() -> Result<Qdrant, Box<dyn std::error::Error>> {
    let config = QdrantConfig::from_url("http://localhost:6334");
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
                config: Some(qdrant_client::qdrant::vectors_config::Config::Params(
                    VectorParams {
                        size: 768, // Adjust this based on your embedding model
                        distance: Distance::Cosine.into(),
                        ..Default::default()
                    },
                )),
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
    let config = QdrantConfig::from_url("http://localhost:6334");
    let client = Qdrant::new(config)?;

    let create_collection = CreateCollection {
        collection_name: "article_embeddings".to_string(),
        vectors_config: Some(VectorsConfig {
            config: Some(qdrant_client::qdrant::vectors_config::Config::Params(
                VectorParams {
                    size: 768,
                    distance: Distance::Cosine.into(),
                    ..Default::default()
                },
            )),
        }),
        ..Default::default()
    };

    client.create_collection(create_collection).await?;

    Ok(client)
}
