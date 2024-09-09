use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::schema::collections;

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::collections)]
pub struct Collection {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub slug: String,
    pub helpscout_collection_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Collection {
    pub fn new(
        name: String,
        description: Option<String>,
        slug: String,
        helpscout_collection_id: String,
    ) -> Self {
        Collection {
            id: Uuid::new_v4(),
            name,
            description,
            slug,
            helpscout_collection_id,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    pub fn store(&self, conn: &mut PgConnection) -> Result<Self, diesel::result::Error> {

        let collection: Self = diesel::insert_into(collections::table)
            .values(self)
            .get_result(conn)
            .expect("Error creating collection");

        Ok(collection)
    }

    // pub fn get_all(conn: &mut PgConnection) -> Result<Vec<Collection>, diesel::result::Error> {
    //     use crate::schema::collections::dsl::*;

    //     collections.load::<Collection>(conn)
    // }
}

impl From<CollectionItem> for Collection {
    fn from(item: CollectionItem) -> Self {
        Collection::new(item.name, item.description, item.slug, item.id)
    }
}

#[derive(Debug, Deserialize)]
pub struct CollectionResponse {
    pub collections: CollectionsData,
}

#[derive(Debug, Deserialize)]
pub struct CollectionsData {
    pub page: i32,
    pub pages: i32,
    pub count: i32,
    pub items: Vec<CollectionItem>,
}

#[derive(Debug, Deserialize)]
pub struct CollectionItem {
    pub id: String,
    #[serde(rename = "siteId")]
    pub site_id: String,
    pub number: i32,
    pub slug: String,
    pub visibility: String,
    pub order: i32,
    pub name: String,
    pub description: Option<String>,
    #[serde(rename = "publicUrl")]
    pub public_url: String,
    #[serde(rename = "articleCount")]
    pub article_count: i32,
    #[serde(rename = "publishedArticleCount")]
    pub published_article_count: i32,
    #[serde(rename = "createdBy")]
    pub created_by: i32,
    #[serde(rename = "updatedBy")]
    pub updated_by: i32,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}
