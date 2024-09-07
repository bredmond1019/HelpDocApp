// File: src/models/embedding.rs

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Embedding {
    pub id: Uuid,
    pub article_id: Uuid,
    pub embedding_vector: Vec<f32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NewEmbedding {
    pub article_id: Uuid,
    pub embedding_vector: Vec<f32>,
}

impl NewEmbedding {
    pub fn new(article_id: Uuid, embedding_vector: Vec<f32>) -> Self {
        NewEmbedding {
            article_id,
            embedding_vector,
        }
    }
}
