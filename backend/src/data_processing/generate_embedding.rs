use anyhow::Result;
use qdrant_client::qdrant::{PointId, PointStruct, Value as QdrantValue, Vectors};
use rust_bert::pipelines::sentence_embeddings::{
    SentenceEmbeddingsBuilder, SentenceEmbeddingsModel, SentenceEmbeddingsModelType,
};
use std::cell::RefCell;
use std::collections::HashMap;
use tokio::task;

use crate::models::{Article, Embedding};

thread_local! {
    static EMBEDDINGS_MODEL: RefCell<Option<SentenceEmbeddingsModel>> = RefCell::new(None);
}

fn get_or_initialize_model() -> &'static SentenceEmbeddingsModel {
    EMBEDDINGS_MODEL.with(|cell| {
        let mut model = cell.borrow_mut();
        if model.is_none() {
            *model = Some(
                SentenceEmbeddingsBuilder::remote(SentenceEmbeddingsModelType::AllMiniLmL12V2)
                    .create_model()
                    .expect("Failed to create SentenceEmbeddingsModel"),
            );
        }
        // This is safe because we never remove the model once it's set
        unsafe { &*(model.as_ref().unwrap() as *const SentenceEmbeddingsModel) }
    })
}

pub async fn generate_embeddings(
    articles: Vec<Article>,
) -> Result<(Vec<Embedding>, Vec<PointStruct>), Box<dyn std::error::Error + Send + Sync>> {
    let embeddings_and_points = task::spawn_blocking(move || {
        let model = get_or_initialize_model();
        let mut embeddings = Vec::new();
        let mut points = Vec::new();

        for article in articles.iter() {
            let title = article.title.clone();
            let embedding = model.encode(&[&title]).expect("Failed to encode article");
            let embedding_vec: Vec<f32> = embedding[0].clone().into();

            let mut payload = HashMap::new();
            payload.insert(
                "article_id".to_string(),
                QdrantValue::from(article.id.to_string()),
            );

            let point = PointStruct {
                id: Some(PointId::from(article.id.to_string())),
                payload,
                vectors: Some(Vectors::from(embedding_vec.clone())),
            };

            points.push(point);
            embeddings.push(Embedding {
                id: article.id,
                article_id: article.id,
                embedding_vector: embedding_vec,
            });
        }
        (embeddings, points)
    })
    .await?;

    Ok(embeddings_and_points)
}
