use chrono::{DateTime, Utc};
use diesel::prelude::*;
use pgvector::Vector;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::schema::collections;

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Insertable, Identifiable)]
#[diesel(table_name = collections)]
pub struct Collection {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub slug: String,
    pub helpscout_collection_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    // Meta Data
    pub paragraph_description: Option<String>,
    pub bullet_points: Option<String>,
    pub keywords: Option<String>,
    pub paragraph_description_embedding: Option<Vector>,
    pub bullet_points_embedding: Option<Vector>,
    pub keywords_embedding: Option<Vector>,
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
            // Meta Data
            paragraph_description: None,
            bullet_points: None,
            keywords: None,
            paragraph_description_embedding: None,
            bullet_points_embedding: None,
            keywords_embedding: None,

        }
    }

    pub fn load_all(conn: &mut PgConnection) -> Result<Vec<Collection>, diesel::result::Error> {
        collections::table.load::<Collection>(conn)
    }

    pub fn store(&self, conn: &mut PgConnection) -> Result<Self, diesel::result::Error> {

        let collection: Self = diesel::insert_into(collections::table)
            .values(self)
            .get_result(conn)
            .expect("Error creating collection");

        Ok(collection)
    }

    pub fn update_metadata(
        &self,
        conn: &mut PgConnection,
        paragraph_description: String,
        bullet_points: String,
        keywords: String,
        paragraph_description_embedding: Vector,
        bullet_points_embedding: Vector,
        keywords_embedding: Vector,
    ) -> Result<(), diesel::result::Error> {
        

        diesel::update(collections::table.find(self.id))
            .set((
                collections::columns::paragraph_description.eq(paragraph_description),
                collections::columns::bullet_points.eq(bullet_points),
                collections::columns::keywords.eq(keywords),
                collections::columns::paragraph_description_embedding.eq(paragraph_description_embedding),
                collections::columns::bullet_points_embedding.eq(bullet_points_embedding),
                collections::columns::keywords_embedding.eq(keywords_embedding),
            ))
            .execute(conn)?;

        Ok(())
    }
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
