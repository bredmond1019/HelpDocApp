// File: src/data_processing/storage.rs

use std::collections::HashMap;

use crate::models::{Article, Collection, Embedding};
use anyhow::Result;

use qdrant_client::qdrant::{
    PointId, PointStruct, UpsertPointsBuilder, Value as QdrantValue, Vectors,
};

use qdrant_client::Qdrant;

use rust_bert::pipelines::sentence_embeddings::{
    SentenceEmbeddingsBuilder, SentenceEmbeddingsModelType,
};

use sqlx::types::time::OffsetDateTime;
use sqlx::PgPool;

pub async fn store_in_postgres(
    pool: &PgPool,
    collection: &Collection,
    articles: &[Article],
) -> Result<()> {
    // Start a transaction
    let mut tx = pool.begin().await?;

    // Insert or update the collection
    sqlx::query!(
        r#"
        INSERT INTO collections (id, name, description, slug, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6)
        ON CONFLICT (id) DO UPDATE
        SET name = EXCLUDED.name,
            description = EXCLUDED.description,
            slug = EXCLUDED.slug,
            updated_at = EXCLUDED.updated_at
        "#,
        collection.id,
        collection.name,
        collection.description,
        collection.slug,
        OffsetDateTime::from_unix_timestamp(collection.created_at.timestamp())?,
        OffsetDateTime::from_unix_timestamp(collection.updated_at.timestamp())?
    )
    .execute(&mut *tx)
    .await?;

    // Insert or update articles
    for article in articles {
        sqlx::query!(
            r#"
            INSERT INTO articles (id, collection_id, title, slug, html_content, markdown_content, version, last_edited_by, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            ON CONFLICT (id) DO UPDATE
            SET collection_id = EXCLUDED.collection_id,
                title = EXCLUDED.title,
                slug = EXCLUDED.slug,
                html_content = EXCLUDED.html_content,
                markdown_content = EXCLUDED.markdown_content,
                version = EXCLUDED.version,
                last_edited_by = EXCLUDED.last_edited_by,
                updated_at = EXCLUDED.updated_at
            "#,
            article.id,
            article.collection_id,
            article.title,
            article.slug,
            article.html_content,
            article.markdown_content,
            article.version,
            article.last_edited_by,
            OffsetDateTime::from_unix_timestamp(article.created_at.timestamp())?,
            OffsetDateTime::from_unix_timestamp(article.updated_at.timestamp())?
        )
        .execute(&mut *tx)
        .await?;
    }

    // Commit the transaction
    tx.commit().await?;

    Ok(())
}

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
            QdrantValue::from(article.id as i64),
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
