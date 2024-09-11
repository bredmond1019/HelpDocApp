// File: src/models/embedding.rs

use diesel::expression::AsExpression;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use diesel::sql_types::VarChar;
use diesel::serialize::{ToSql, Output};
use diesel::deserialize::{FromSql, FromSqlRow, Result as DeserializeResult};
use std::io::Write;

#[derive(Debug, Clone, Serialize, Deserialize, AsExpression, FromSqlRow)]
#[diesel(sql_type = VarChar)]
pub struct Vector(pub Vec<f32>);

impl ToSql<VarChar, diesel::pg::Pg> for Vector {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, diesel::pg::Pg>) -> diesel::serialize::Result {
        let vector_string = self.0.iter()
            .map(|&f| f.to_string())
            .collect::<Vec<String>>()
            .join(",");
        write!(out, "[{}]", vector_string)?;
        Ok(diesel::serialize::IsNull::No)
    }
}

impl FromSql<VarChar, diesel::pg::Pg> for Vector {
    fn from_sql(bytes: diesel::pg::PgValue) -> DeserializeResult<Self> {
        let s = String::from_utf8(bytes.as_bytes().to_vec())?;
        let s = s.trim_matches(|c| c == '[' || c == ']');
        let vec: Vec<f32> = s.split(',')
            .map(|s| s.parse().unwrap())
            .collect();
        Ok(Vector(vec))
    }
}

#[derive(Debug, Clone, Insertable, Queryable)]
#[diesel(table_name = crate::schema::embeddings)]
pub struct Embedding {
    pub id: Uuid,
    pub article_id: Uuid,
    pub embedding_vector: Vector,
}

impl Embedding {
    pub fn new(article_id: Uuid, embedding_vector: Vec<f32>) -> Self {
        assert_eq!(embedding_vector.len(), 384, "Embedding vector must be 384 dimensions");
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
