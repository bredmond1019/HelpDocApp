// File: src/models/embedding.rs

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Embedding {
    pub id: i32,
    pub article_id: i32,
    pub embedding_vector: Vec<f32>,
}

// TODO: Implement any necessary methods for the Embedding struct
