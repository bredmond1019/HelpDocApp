use anyhow::Result;
use log::info;
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
    let embedding_and_point = task::spawn_blocking(move || -> Result<_, Box<dyn std::error::Error + Send + Sync>> {
        let model = get_or_initialize_model();
        
        let content_to_encode = article.html_content.as_deref().unwrap_or(&article.title);
        
        let embedding = model.encode(&[content_to_encode])
            .map_err(|e| anyhow::anyhow!("Failed to encode article: {}", e))?;
        
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
        info!("Point: {:?}", point);
        info!("Embedding Vec: {:?}", embedding_vec);
        info!("Vectors: {:?}", point.vectors);

        let embedding = Embedding {
            id: Uuid::new_v4(),
            article_id: article.id,
            embedding_vector: embedding_vec,
        };

        Ok((embedding, point))
    })
    .await?;

    match embedding_and_point { 
        Ok((embedding, point)) => Ok((embedding, point)),
        Err(e) => Err(e),
    }
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
