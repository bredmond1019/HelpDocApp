// File: src/data_processing/storage.rs

use anyhow::Result;
use qdrant_client::qdrant::{
    PointId, PointStruct, UpsertPointsBuilder, Value as QdrantValue, Vectors,
};
use qdrant_client::Qdrant;
use rust_bert::pipelines::sentence_embeddings::{
    SentenceEmbeddingsBuilder, SentenceEmbeddingsModelType,
};
use std::collections::HashMap;

use crate::models::{Article, Embedding};

pub async fn generate_embeddings(
    client: &Qdrant,
    articles: &[Article],
) -> Result<Vec<Embedding>, Box<dyn std::error::Error>> {
    let model = SentenceEmbeddingsBuilder::remote(SentenceEmbeddingsModelType::AllMiniLmL12V2)
        .create_model()?;

    let mut embeddings = Vec::new();

    for article in articles {
        let embedding = model.encode(&[&article.title])?;
        let embedding_vec: Vec<f32> = embedding[0].clone().into();

        let mut payload = HashMap::new();
        payload.insert(
            "article_id".to_string(),
            QdrantValue::from(article.id.to_string()),
        );

        let point = PointStruct {
            id: Some(PointId::from(article.id.to_string())),
            payload: payload,
            vectors: Some(Vectors::from(embedding_vec.clone())),
        };

        let points = vec![point];

        client
            .upsert_points(UpsertPointsBuilder::new("article_embeddings", points))
            .await?;

        embeddings.push(Embedding {
            id: article.id,
            article_id: article.id,
            embedding_vector: embedding_vec,
        });
    }

    Ok(embeddings)
}
