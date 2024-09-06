use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Collection {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub slug: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Collection {
    pub fn new(name: String, description: Option<String>, slug: String) -> Self {
        Collection {
            id: 0,
            name,
            description,
            slug,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}
