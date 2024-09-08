use anyhow::Result;
use qdrant_client::qdrant::{
    PointId, PointStruct, UpsertPointsBuilder, Value as QdrantValue, Vectors,
};
use rust_bert::pipelines::sentence_embeddings::{
    SentenceEmbeddingsBuilder, SentenceEmbeddingsModel, SentenceEmbeddingsModelType,
};
use std::collections::HashMap;
use std::{cell::RefCell, sync::Arc};
use tokio::task;
use uuid::Uuid;

use crate::db::DbPool;
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
    article: Article,
) -> Result<(Embedding, PointStruct), Box<dyn std::error::Error + Send + Sync>> {
    let embedding_and_point = task::spawn_blocking(move || {
        let model = get_or_initialize_model();
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

        let embedding = Embedding {
            id: Uuid::new_v4(),
            article_id: article.id,
            embedding_vector: embedding_vec,
        };

        (embedding, point)
    })
    .await?;

    Ok(embedding_and_point)
}

pub async fn store_embedding(
    embedding_and_point: (Embedding, PointStruct),
    vector_db_client: Arc<qdrant_client::Qdrant>,
    db_pool: Arc<DbPool>,
) -> Result<()> {
    let (embedding, point) = embedding_and_point;

    vector_db_client
        .upsert_points(UpsertPointsBuilder::new("article_embeddings", vec![point]))
        .await?;

    embedding
        .store(&mut db_pool.get().expect("Failed to get DB connection"))
        .expect("Failed to store embedding");

    Ok(())
}
