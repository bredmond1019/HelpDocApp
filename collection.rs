use chrono::{DateTime, Utc};
use diesel::prelude::*;
use pgvector::Vector;
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
    // New fields
    pub paragraph_description: Option<String>,
    pub bullet_points: Option<String>,
    pub keywords: Option<String>,
    pub paragraph_description_embedding: Option<Vector>,
    pub bullet_points_embedding: Option<Vector>,
    pub keywords_embedding: Option<Vector>,
}

impl Collection {
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
        use crate::schema::collections::dsl::*;

        diesel::update(collections.find(self.id))
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