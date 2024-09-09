use chrono::{DateTime, Utc};
use diesel::prelude::*;
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
        }
    }

    pub fn store(&self, conn: &mut PgConnection) -> Result<(), diesel::result::Error> {
        use crate::schema::articles::dsl::*;

        diesel::insert_into(articles)
            .values(self)
            .on_conflict(id)
            .do_update()
            .set((
                collection_id.eq(self.collection_id),
                title.eq(&self.title),
                slug.eq(&self.slug),
                html_content.eq(&self.html_content),
                markdown_content.eq(&self.markdown_content),
                version.eq(self.version),
                last_edited_by.eq(&self.last_edited_by),
                updated_at.eq(self.updated_at),
                helpscout_collection_id.eq(&self.helpscout_collection_id),
                helpscout_article_id.eq(&self.helpscout_article_id),
            ))
            .execute(conn)?;

        Ok(())
    }

    pub fn get_by_id(
        conn: &mut PgConnection,
        article_id: Uuid,
    ) -> Result<Option<Article>, diesel::result::Error> {
        use crate::schema::articles::dsl::*;

        articles.find(article_id).first(conn).optional()
    }

    pub fn update_markdown_content(
        &self,
        conn: &mut PgConnection,
        markdown: String,
    ) -> Result<(), diesel::result::Error> {
        use crate::schema::articles::dsl::*;

        diesel::update(articles.find(self.id))
            .set(markdown_content.eq(markdown))
            .execute(conn)?;

        Ok(())
    }
}

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
    pub last_published_at: String,
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
    pub last_published_at: String,
}
