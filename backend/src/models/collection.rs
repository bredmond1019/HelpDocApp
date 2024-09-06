use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Collection {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub slug: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub helpscout_collection_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NewCollection {
    pub name: String,
    pub description: Option<String>,
    pub slug: String,
    pub helpscout_collection_id: String,
}

impl NewCollection {
    pub fn new(
        name: String,
        description: Option<String>,
        slug: String,
        helpscout_collection_id: String,
    ) -> Self {
        NewCollection {
            name,
            description,
            slug,
            helpscout_collection_id,
        }
    }
}
