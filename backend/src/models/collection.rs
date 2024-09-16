use chrono::{DateTime, Utc};
use diesel::{
    dsl::{sql, Nullable},
    prelude::*,
    sql_query,
    sql_types::{Array, Double, SingleValue, Text, Timestamptz},
};
use pgvector::{Vector, VectorExpressionMethods};
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

    pub fn store(&self, conn: &mut PgConnection) -> Result<(), diesel::result::Error> {
        diesel::insert_into(collections::table)
            .values(self)
            .execute(conn)?;
        Ok(())
    }

    pub fn find_relevant_collections(
        query_embedding: &Vector,
        conn: &mut PgConnection,
    ) -> Result<Vec<(Collection, f64)>, diesel::result::Error> {
        let collection_table = collections::table;

        let paragraph_description_embedding: Vec<(Collection, Option<f64>)> = collection_table
            .select((
                collections::all_columns,
                collections::paragraph_description_embedding
                    .cosine_distance(query_embedding)
                    .nullable(),
            ))
            .filter(collections::paragraph_description_embedding.is_not_null())
            .order(collections::paragraph_description_embedding.cosine_distance(query_embedding))
            .limit(3)
            .load::<(Collection, Option<f64>)>(conn)?;

        let bullet_points_embedding: Vec<(Collection, Option<f64>)> = collection_table
            .select((
                collections::all_columns,
                collections::bullet_points_embedding.cosine_distance(query_embedding),
            ))
            .filter(collections::bullet_points_embedding.is_not_null())
            .order(collections::bullet_points_embedding.cosine_distance(query_embedding))
            .limit(3)
            .load::<(Collection, Option<f64>)>(conn)?;

        let keywords_embedding: Vec<(Collection, Option<f64>)> = collection_table
            .select((
                collections::all_columns,
                collections::keywords_embedding.cosine_distance(query_embedding),
            ))
            .filter(collections::keywords_embedding.is_not_null())
            .order(collections::keywords_embedding.cosine_distance(query_embedding))
            .limit(3)
            .load::<(Collection, Option<f64>)>(conn)?;

        // Combine all results
        let mut combined_results: Vec<(Collection, f64)> = paragraph_description_embedding
            .into_iter()
            .chain(bullet_points_embedding)
            .chain(keywords_embedding)
            .filter_map(|(collection, distance)| distance.map(|d| (collection, d)))
            .collect();

        // Sort by distance (lower is better)
        combined_results.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        // Take the top 3 unique results
        let mut unique_results = Vec::new();
        let mut seen_ids = std::collections::HashSet::new();

        for (collection, distance) in combined_results {
            if seen_ids.insert(collection.id) {
                unique_results.push((collection, distance));
                if unique_results.len() == 3 {
                    break;
                }
            }
        }

        Ok(unique_results)
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
                collections::columns::paragraph_description_embedding
                    .eq(paragraph_description_embedding),
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

use diesel::sql_types::*;

#[derive(QueryableByName, Debug)]
#[diesel(table_name = collections)]
struct CollectionWithDistance {
    #[diesel(sql_type = Uuid)]
    id: uuid::Uuid,
    #[diesel(sql_type = Text)]
    name: String,
    #[diesel(sql_type = Nullable<Text>)]
    description: Option<String>,
    #[diesel(sql_type = Text)]
    slug: String,
    #[diesel(sql_type = Text)]
    helpscout_collection_id: String,
    #[diesel(sql_type = Timestamptz)]
    created_at: chrono::DateTime<chrono::Utc>,
    #[diesel(sql_type = Timestamptz)]
    updated_at: chrono::DateTime<chrono::Utc>,
    #[diesel(sql_type = Nullable<Text>)]
    paragraph_description: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    bullet_points: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    keywords: Option<String>,
    #[diesel(sql_type = Nullable<Array<Float8>>)]
    paragraph_description_embedding: Option<Vec<f32>>,
    #[diesel(sql_type = Nullable<Array<Float8>>)]
    bullet_points_embedding: Option<Vec<f32>>,
    #[diesel(sql_type = Nullable<Array<Float8>>)]
    keywords_embedding: Option<Vec<f32>>,
    #[diesel(sql_type = Float8)]
    distance: f64,
}
