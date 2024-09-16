use chrono::{DateTime, Utc};
use diesel::associations::HasTable;
use diesel::prelude::*;
use diesel::BelongingToDsl;
use pgvector::Vector;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::Collection;

use crate::schema::articles;

pub mod article_chunk;
pub mod parse;
pub mod query;
pub mod update;

pub use self::article_chunk::*;
pub use self::parse::*;

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Insertable, Associations)]
#[diesel(table_name = crate::schema::articles)]
#[diesel(belongs_to(Collection, foreign_key = collection_id))]
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
    // Meta Data
    pub paragraph_description: Option<String>,
    pub bullet_points: Option<String>,
    pub keywords: Option<String>,
    pub paragraph_description_embedding: Option<Vector>,
    pub bullet_points_embedding: Option<Vector>,
    pub keywords_embedding: Option<Vector>,
}

impl Article {
    pub fn new(
        collection_id: Uuid,
        helpscout_collection_id: String,
        helpscout_article_id: Option<String>,
        title: String,
        slug: String,
        html_content: Option<String>,
    ) -> Self {
        Article {
            id: Uuid::new_v4(),
            collection_id,
            helpscout_collection_id,
            helpscout_article_id,
            title,
            slug,
            html_content,
            markdown_content: None,
            version: 0,
            last_edited_by: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            paragraph_description: None,
            bullet_points: None,
            keywords: None,
            paragraph_description_embedding: None,
            bullet_points_embedding: None,
            keywords_embedding: None,
        }
    }

    pub fn ids_from_articles(articles: &[Article]) -> Vec<Uuid> {
        articles.iter().map(|article| article.id).collect()
    }

    pub fn get_by_id(
        conn: &mut PgConnection,
        article_id: Uuid,
    ) -> Result<Option<Article>, diesel::result::Error> {
        use crate::schema::articles::dsl::*;

        articles.find(article_id).first(conn).optional()
    }

    pub fn belonging_to_collection(
        collection: &Collection,
        conn: &mut PgConnection,
    ) -> Result<Vec<Article>, diesel::result::Error> {
        Article::belonging_to(collection).load::<Article>(conn)
    }

    pub fn article_ids_by_collection(
        collection: &Collection,
        conn: &mut PgConnection,
    ) -> Result<Vec<Uuid>, diesel::result::Error> {
        Article::belonging_to(collection)
            .select(articles::id)
            .load::<Uuid>(conn)
    }

    pub fn load_all(conn: &mut PgConnection) -> Result<Vec<Article>, diesel::result::Error> {
        articles::table.load::<Article>(conn)
    }

    pub fn store(&self, conn: &mut PgConnection) -> Result<Self, diesel::result::Error> {
        log::info!("Storing article: ID:{:?}, Title: {:?}", self.id, self.title);

        let article: Self = diesel::insert_into(articles::table)
            .values(self)
            .get_result(conn)
            .expect("Error creating article");
        log::info!(
            "Result: Article ID: {:?}, Article Title: {:?}",
            article.id,
            article.title
        );

        Ok(article)
    }
}

impl HasTable for Article {
    type Table = articles::table;

    fn table() -> Self::Table {
        articles::table
    }
}
