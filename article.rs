use chrono::{DateTime, Utc};
use diesel::prelude::*;
use pgvector::Vector;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::articles)]
pub struct Article {
    pub id: Uuid,
    pub collection_id: Uuid,
    pub title: String,
    pub slug: String,
    pub html_content: Option<String>,
    pub markdown_content: Option<String>,
    pub version: i32,
    pub last_edited_by: Option<String>,
    pub helpscout_collection_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub helpscout_article_id: Option<String>,
    // New fields
    pub paragraph_description: Option<String>,
    pub bullet_points: Option<String>,
    pub keywords: Option<String>,
    pub paragraph_description_embedding: Option<Vector>,
    pub bullet_points_embedding: Option<Vector>,
    pub keywords_embedding: Option<Vector>,
}

// ... (rest of the implementation remains the same)

impl Article {
    // ... (existing methods)

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
        use crate::schema::articles::dsl::*;

        diesel::update(articles.find(self.id))
            .set((
                paragraph_description.eq(paragraph_description),
                bullet_points.eq(bullet_points),
                keywords.eq(keywords),
                paragraph_description_embedding.eq(paragraph_description_embedding),
                bullet_points_embedding.eq(bullet_points_embedding),
                keywords_embedding.eq(keywords_embedding),
            ))
            .execute(conn)?;

        Ok(())
    }
}