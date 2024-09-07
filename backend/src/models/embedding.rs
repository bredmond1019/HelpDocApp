// File: src/models/embedding.rs

use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Queryable, Insertable)]
#[diesel(table_name = crate::schema::embeddings)]
pub struct Embedding {
    pub id: Uuid,
    pub article_id: Uuid,
    #[diesel(sql_type = Array<Float4>)]
    pub embedding_vector: Vec<f32>,
}

impl Embedding {
    pub fn new(article_id: Uuid, embedding_vector: Vec<f32>) -> Self {
        Self {
            id: Uuid::new_v4(),
            article_id,
            embedding_vector,
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
            .execute(conn)?;

        Ok(())
    }
}
