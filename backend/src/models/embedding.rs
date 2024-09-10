// File: src/models/embedding.rs

use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::vector::Vector;

#[derive(Debug, Serialize, Deserialize, Queryable, Insertable)]
#[diesel(table_name = crate::schema::embeddings)]
pub struct Embedding {
    pub id: Uuid,
    pub article_id: Uuid,
    #[diesel(sql_type = VarChar)]
    pub embedding_vector: Vector,
}

impl Embedding {
    pub fn new(article_id: Uuid, embedding_vector: Vec<f32>) -> Self {
        Self {
            id: Uuid::new_v4(),
            article_id,
            embedding_vector: Vector(embedding_vector),
        }
    }

    pub fn store(&self, conn: &mut PgConnection) -> Result<(), diesel::result::Error> {
        use crate::schema::embeddings::dsl::*;

        diesel::insert_into(embeddings)
            .values(self)
            .on_conflict(id)
            .do_update()
            .set((
                article_id.eq(self.article_id),
                embedding_vector.eq(&self.embedding_vector),
            ))
            .execute(conn)
            .map_err(|e| {
                log::error!("Failed to store embedding in database: {}", e);
                e
            })?;

        log::info!("Successfully stored embedding for article {}", self.article_id);
        Ok(())
    }
}
