use std::collections::HashMap;

use chrono::{DateTime, Utc};
use diesel::debug_query;
use diesel::prelude::*;
use diesel::sql_types::Integer;
use pgvector::Vector;
use pgvector::VectorExpressionMethods;
use diesel::ExpressionMethods;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use log::info;

use crate::schema::article_chunks;
use crate::schema::articles;

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

    pub fn store(&self, conn: &mut PgConnection) -> Result<Self, diesel::result::Error> {
        log::info!("Storing article: ID:{:?}, Title: {:?}", self.id, self.title);

        let article: Self = diesel::insert_into(articles::table)
            .values(self)
            .get_result(conn)
            .expect("Error creating article");
        log::info!("Result: Article ID: {:?}, Article Title: {:?}", article.id, article.title);

        Ok(article)
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

    pub fn create_chunks(&self, chunk_size: usize) -> Vec<ArticleChunk> {
        let mut chunks = Vec::new();
        
        // Add title as a separate chunk
        let title_chunk = ArticleChunk {
            id: Uuid::new_v4(),
            article_id: self.id,
            content: self.title.clone(),
            is_title: true,
            embedding_id: None,
        };
        chunks.push(title_chunk);
    
        if let Some(content) = &self.markdown_content {
            // Split content into words
            let words: Vec<&str> = content.split_whitespace().collect();
            let mut current_chunk = String::new();
            let mut word_count = 0;
    
            for word in words {
                if word_count >= chunk_size {
                    // If the current chunk has reached or exceeded the chunk size,
                    // add it to the chunks vector and start a new chunk
                    chunks.push(ArticleChunk {
                        id: Uuid::new_v4(),
                        article_id: self.id,
                        content: current_chunk.trim().to_string(),
                        is_title: false,
                        embedding_id: None,
                    });
                    current_chunk.clear();
                    word_count = 0;
                }
    
                // Add the word to the current chunk
                if !current_chunk.is_empty() {
                    current_chunk.push(' ');
                }
                current_chunk.push_str(word);
                word_count += 1;
            }
    
            // Add any remaining content as the last chunk
            if !current_chunk.is_empty() {
                chunks.push(ArticleChunk {
                    id: Uuid::new_v4(),
                    article_id: self.id,
                    content: current_chunk.trim().to_string(),
                    is_title: false,
                    embedding_id: None,
                });
            }
        }
    
        chunks
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

        diesel::update(articles::table.find(self.id))
            .set((
                articles::columns::paragraph_description.eq(paragraph_description),
                articles::columns::bullet_points.eq(bullet_points),
                articles::columns::keywords.eq(keywords),
                articles::columns::paragraph_description_embedding.eq(paragraph_description_embedding),
                articles::columns::bullet_points_embedding.eq(bullet_points_embedding),
                articles::columns::keywords_embedding.eq(keywords_embedding),
            ))
            .execute(conn)?;

        Ok(())
    }

    pub async fn find_relevant_articles(
        query_embedding: &Vector,
        conn: &mut PgConnection,
    ) -> Result<Vec<(Article, f64)>, Box<dyn std::error::Error + Send + Sync>> {
        info!("Finding relevant articles based on query embedding");
        use diesel::prelude::*;
        use crate::schema::{articles, article_chunks, embeddings};

        let results: Vec<(Article, f64, bool)> = article_chunks::table
            .inner_join(articles::table.on(articles::id.eq(article_chunks::article_id)))
            .inner_join(embeddings::table.on(embeddings::id.nullable().eq(article_chunks::embedding_id)))
            .select((
                articles::all_columns,
                embeddings::embedding_vector.cosine_distance(query_embedding),
                article_chunks::is_title,
            ))
            .order(embeddings::embedding_vector.cosine_distance(query_embedding))
            .limit(20)
            .load(conn)?;

        // Group by article and calculate weighted similarity
        let mut article_similarities: HashMap<Uuid, (Article, f64)> = HashMap::new();
        for (article, distance, is_title) in results {
            let similarity = 1.0 - distance;
            let weight = if is_title { 2.0 } else { 1.0 };
            let entry = article_similarities.entry(article.id).or_insert_with(|| (article, 0.0));
            entry.1 += similarity * weight;
        }

        let mut sorted_articles: Vec<(Article, f64)> = article_similarities.into_values().collect();
        sorted_articles.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        sorted_articles.truncate(5);

        info!("Found {} relevant articles", sorted_articles.len());
        Ok(sorted_articles)
    }

    pub fn keyword_search(conn: &mut PgConnection, query: &str) -> Result<Vec<Article>, diesel::result::Error> {
        info!("Performing keyword search for query: {}", query);
        let words: Vec<String> = query.split_whitespace().map(|w| w.to_lowercase()).collect();
        
        // Create the base query
        let mut query = articles::table.into_boxed();

        // Add ILIKE conditions for each word
        for word in &words {
            info!("Adding ILIKE condition for word: {}", word);
            let like_word = format!("%{}%", word);
            query = query.filter(
                articles::title.ilike(like_word.clone()).or(articles::markdown_content.ilike(like_word))
            );
        }

        // Add ordering
        query = query.order(
            (
                diesel::dsl::sql::<Integer>(&format!(
                    "{}",
                    words.iter()
                        .map(|w| format!("CASE WHEN LOWER(title) LIKE '%{}%' THEN 1 ELSE 0 END", w))
                        .collect::<Vec<_>>()
                        .join(" + ")
                ))
            ).desc()
        ).then_order_by(articles::updated_at.desc());

        // Execute the query
        let results = query.limit(10).load::<Article>(conn)?;
        info!("Keyword search found {} results", results.len());
        Ok(results)
    }

    pub fn keyword_search_with_ids(
        conn: &mut PgConnection,
        query: &str,
        ids: &[Uuid],
    ) -> Result<Vec<Article>, diesel::result::Error> {
        info!("Performing keyword search with IDs for query: {}", query);
        use crate::schema::articles::dsl::*;

        let words: Vec<String> = query.split_whitespace().map(|w| w.to_lowercase()).collect();

        info!("Words: {:?}", words);
        
        let mut query = articles.into_boxed();

        // Add ILIKE conditions for each word
        for word in &words {
            let like_word = format!("%{}%", word);
            query = query.filter(
                title.ilike(like_word.clone()).or(markdown_content.ilike(like_word))
            );
        }

        info!("Query after adding ILIKE conditions: {:?}", debug_query(&query));

        // Add filter for specific IDs
        query = query.filter(id.eq_any(ids));

        info!("Query after adding filter for specific IDs: {:?}", debug_query(&query));

        // Add ordering
        query = query.order(
            (
                diesel::dsl::sql::<Integer>(&format!(
                    "{}",
                    words.iter()
                        .map(|w| format!("CASE WHEN LOWER(title) LIKE '%{}%' THEN 1 ELSE 0 END", w))
                        .collect::<Vec<_>>()
                        .join(" + ")
                ))
            ).desc()
        ).then_order_by(updated_at.desc());

        info!("Query after adding ordering: {:?}", debug_query(&query));

        // Execute the query
        let results = query.limit(10).load::<Article>(conn)?;
        info!("Keyword search with IDs found {} results", results.len());
        Ok(results)
    }
}


#[derive(Queryable, Insertable)]
#[diesel(table_name = crate::schema::article_chunks)]
pub struct ArticleChunk {
    pub id: Uuid,
    pub article_id: Uuid,
    pub content: String,
    pub is_title: bool,
    pub embedding_id: Option<Uuid>,
}

impl ArticleChunk {
    pub fn store(&self, conn: &mut PgConnection) -> Result<Self, diesel::result::Error> {
        let chunk: Self = diesel::insert_into(article_chunks::table)
            .values(self)
            .get_result(conn)?;
        Ok(chunk)
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
