// File: src/models/embedding.rs


use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use pgvector::Vector as PgVector;
use diesel::pg::PgConnection;

use crate::schema::embeddings;


#[derive(Insertable, Queryable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::embeddings)]
pub struct Embedding {
    pub id: Uuid,
    pub article_id: Uuid,
    pub embedding_vector: PgVector,
}

impl Embedding {
    pub fn new(article_id: Uuid, embedding_vector: Vec<f32>) -> Self {
        assert_eq!(embedding_vector.len(), 384, "Embedding vector must be 384 dimensions");
        let embedding = PgVector::from(embedding_vector);
        Self {
            id: Uuid::new_v4(),
            article_id,
            embedding_vector: embedding,
        }
    }

    pub fn store(&self, conn: &mut PgConnection) -> Result<Self, diesel::result::Error> {
        let embedding: Self = diesel::insert_into(embeddings::table)
        .values(self)
        .get_result(conn)?;

        log::info!("Successfully stored embedding for article {}", self.article_id);
        Ok(embedding)
    }

    pub fn get_failed_embeddings(conn: &mut PgConnection) -> Result<Vec<Embedding>, diesel::result::Error> {
        let failed_embeddings = embeddings::table
            .filter(embeddings::embedding_vector.is_null())
            .load::<Embedding>(conn)?;
        Ok(failed_embeddings)
    }
}
