use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct ArticleResponse {
    pub articles: ArticleData,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ArticleData {
    pub page: i32,
    pub pages: i32,
    pub count: i32,
    pub items: Vec<ArticleRef>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ArticleRef {
    pub id: String,
    pub number: i32,
    #[serde(rename = "collectionId")]
    pub collection_id: String,
    pub status: String,
    #[serde(rename = "hasDraft")]
    pub has_draft: bool,
    pub name: String,
    #[serde(rename = "publicUrl")]
    pub public_url: String,
    pub popularity: f64,
    #[serde(rename = "viewCount")]
    pub view_count: i32,
    #[serde(rename = "createdBy")]
    pub created_by: i32,
    #[serde(rename = "updatedBy")]
    pub updated_by: Option<i32>,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: Option<String>,
    #[serde(rename = "lastPublishedAt")]
    pub last_published_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ArticleFullResponse {
    pub article: ArticleFull,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ArticleFull {
    pub id: String,
    pub number: i32,
    #[serde(rename = "collectionId")]
    pub collection_id: String,
    pub slug: String,
    pub status: String,
    #[serde(rename = "hasDraft")]
    pub has_draft: bool,
    pub name: String,
    pub text: String,
    pub categories: Vec<String>,
    pub related: Option<Vec<String>>,
    #[serde(rename = "publicUrl")]
    pub public_url: String,
    pub popularity: f64,
    #[serde(rename = "viewCount")]
    pub view_count: i32,
    #[serde(rename = "createdBy")]
    pub created_by: i32,
    #[serde(rename = "updatedBy")]
    pub updated_by: i32,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
    #[serde(rename = "lastPublishedAt")]
    pub last_published_at: Option<String>,
}
