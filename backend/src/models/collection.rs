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
