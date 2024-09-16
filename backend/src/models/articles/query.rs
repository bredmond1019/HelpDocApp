use diesel::debug_query;
use diesel::prelude::*;
use diesel::sql_types::Integer;
use diesel::ExpressionMethods;
use log::info;
use pgvector::Vector;
use pgvector::VectorExpressionMethods;
use std::collections::HashMap;
use uuid::Uuid;

use super::Article;
use crate::schema::articles;

impl Article {
    pub async fn find_relevant_articles_by_ids(
        query_embedding: &Vector,
        conn: &mut PgConnection,
        article_ids: &[Uuid],
    ) -> Result<Vec<(Article, f64)>, Box<dyn std::error::Error + Send + Sync>> {
        use crate::schema::{article_chunks, articles, embeddings};
        use diesel::prelude::*;

        info!("Finding relevant articles based on query embedding and article IDs");

        let results: Vec<(Article, f64, bool)> = article_chunks::table
            .inner_join(articles::table.on(articles::id.eq(article_chunks::article_id)))
            .inner_join(
                embeddings::table.on(embeddings::id.nullable().eq(article_chunks::embedding_id)),
            )
            .filter(articles::id.eq_any(article_ids))
            .select((
                articles::all_columns,
                embeddings::embedding_vector.cosine_distance(query_embedding),
                article_chunks::is_title,
            ))
            .order(embeddings::embedding_vector.cosine_distance(query_embedding))
            .limit(20)
            .load(conn)?;

        process_results(results)
    }

    pub async fn find_relevant_articles(
        query_embedding: &Vector,
        conn: &mut PgConnection,
    ) -> Result<Vec<(Article, f64)>, Box<dyn std::error::Error + Send + Sync>> {
        info!("Finding relevant articles based on query embedding");
        use crate::schema::{article_chunks, articles, embeddings};
        use diesel::prelude::*;

        let results: Vec<(Article, f64, bool)> = article_chunks::table
            .inner_join(articles::table.on(articles::id.eq(article_chunks::article_id)))
            .inner_join(
                embeddings::table.on(embeddings::id.nullable().eq(article_chunks::embedding_id)),
            )
            .select((
                articles::all_columns,
                embeddings::embedding_vector.cosine_distance(query_embedding),
                article_chunks::is_title,
            ))
            .order(embeddings::embedding_vector.cosine_distance(query_embedding))
            .limit(20)
            .load(conn)?;

        // Group by article and calculate weighted similarity
        process_results(results)
    }

    pub fn keyword_search(
        conn: &mut PgConnection,
        query: &str,
    ) -> Result<Vec<Article>, diesel::result::Error> {
        info!("Performing keyword search for query: {}", query);
        let words: Vec<String> = query.split_whitespace().map(|w| w.to_lowercase()).collect();

        // Create the base query
        let mut query = articles::table.into_boxed();

        // Add ILIKE conditions for each word
        for word in &words {
            info!("Adding ILIKE condition for word: {}", word);
            let like_word = format!("%{}%", word);
            query = query.filter(
                articles::title
                    .ilike(like_word.clone())
                    .or(articles::markdown_content.ilike(like_word)),
            );
        }

        // Add ordering
        query = query
            .order(
                (diesel::dsl::sql::<Integer>(&format!(
                    "{}",
                    words
                        .iter()
                        .map(|w| format!("CASE WHEN LOWER(title) LIKE '%{}%' THEN 1 ELSE 0 END", w))
                        .collect::<Vec<_>>()
                        .join(" + ")
                )))
                .desc(),
            )
            .then_order_by(articles::updated_at.desc());

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
                title
                    .ilike(like_word.clone())
                    .or(markdown_content.ilike(like_word)),
            );
        }

        info!(
            "Query after adding ILIKE conditions: {:?}",
            debug_query(&query)
        );

        // Add filter for specific IDs
        query = query.filter(id.eq_any(ids));

        info!(
            "Query after adding filter for specific IDs: {:?}",
            debug_query(&query)
        );

        // Add ordering
        query = query
            .order(
                (diesel::dsl::sql::<Integer>(&format!(
                    "{}",
                    words
                        .iter()
                        .map(|w| format!("CASE WHEN LOWER(title) LIKE '%{}%' THEN 1 ELSE 0 END", w))
                        .collect::<Vec<_>>()
                        .join(" + ")
                )))
                .desc(),
            )
            .then_order_by(updated_at.desc());

        info!("Query after adding ordering: {:?}", debug_query(&query));

        // Execute the query
        let results = query.limit(10).load::<Article>(conn)?;
        info!("Keyword search with IDs found {} results", results.len());
        Ok(results)
    }
}

fn process_results(
    results: Vec<(Article, f64, bool)>,
) -> Result<Vec<(Article, f64)>, Box<dyn std::error::Error + Send + Sync>> {
    let mut article_similarities: HashMap<Uuid, (Article, f64)> = HashMap::new();
    for (article, distance, is_title) in results {
        let similarity = 1.0 - distance;
        let weight = if is_title { 2.0 } else { 1.0 };
        let entry = article_similarities
            .entry(article.id)
            .or_insert_with(|| (article, 0.0));
        entry.1 += similarity * weight;
    }

    let mut sorted_articles: Vec<(Article, f64)> = article_similarities.into_values().collect();
    sorted_articles.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    sorted_articles.truncate(5);

    info!("Found {} relevant articles", sorted_articles.len());
    Ok(sorted_articles)
}
