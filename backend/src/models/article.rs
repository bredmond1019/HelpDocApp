use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Article {
    pub id: Uuid,
    pub collection_id: Uuid,
    pub title: String,
    pub slug: String,
    pub html_content: Option<String>,
    pub markdown_content: Option<String>,
    pub version: i32,
    pub last_edited_by: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub helpscout_collection_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NewArticle {
    pub collection_id: Uuid,
    pub title: String,
    pub slug: String,
    pub html_content: Option<String>,
    pub helpscout_collection_id: String,
}

impl NewArticle {
    pub fn new(
        collection_id: Uuid,
        title: String,
        slug: String,
        html_content: Option<String>,
        helpscout_collection_id: String,
    ) -> Self {
        NewArticle {
            collection_id,
            title,
            slug,
            html_content,
            helpscout_collection_id,
        }
    }

    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ArticleJson {
    pub id: String,
    #[serde(rename = "collectionId")]
    pub collection_id: String,
    pub title: String,
    pub slug: String,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
    #[serde(rename = "lastPublishedAt")]
    pub last_published_at: String,
}
