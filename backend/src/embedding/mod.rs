use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use reqwest;

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

    pub async fn generate_embedding(text: &str) -> Result<Vec<f32>, reqwest::Error> {
        let client = reqwest::Client::new();
        let response: serde_json::Value = client.post("http://localhost:5000/embed")
            .json(&serde_json::json!({ "text": text }))
            .send()
            .await?
            .json()
            .await?;

        let embedding = response["embedding"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_f64().unwrap() as f32)
            .collect();

        Ok(embedding)
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