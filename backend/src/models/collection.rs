use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::collections)]
pub struct Collection {
    pub id: Uuid,
    pub helpscout_collection_id: String,
    pub name: String,
    pub description: Option<String>,
    pub slug: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Collection {
    pub fn new(
        helpscout_collection_id: String,
        name: String,
        description: Option<String>,
        slug: String,
    ) -> Self {
        Collection {
            id: Uuid::new_v4(),
            name,
            description,
            slug,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            helpscout_collection_id,
        }
    }

    pub fn store(&self, conn: &mut PgConnection) -> Result<(), diesel::result::Error> {
        use crate::schema::collections::dsl::*;

        diesel::insert_into(collections)
            .values(self)
            .on_conflict(id)
            .do_update()
            .set((
                name.eq(&self.name),
                description.eq(&self.description),
                slug.eq(&self.slug),
                updated_at.eq(self.updated_at),
            ))
            .execute(conn)?;

        Ok(())
    }
}

impl From<CollectionItem> for Collection {
    fn from(item: CollectionItem) -> Self {
        Collection::new(item.id, item.name, item.description, item.slug)
    }
}

#[derive(Debug, Deserialize)]
pub struct CollectionResponse {
    pub collection_data: CollectionsData,
}

#[derive(Debug, Deserialize)]
pub struct CollectionsData {
    pub page: i32,
    pub pages: i32,
    pub count: i32,
    pub collections: Vec<CollectionItem>,
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
