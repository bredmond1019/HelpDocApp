use diesel::prelude::*;
use diesel::sql_types::Integer;
use diesel::ExpressionMethods;
use log::info;
use pgvector::Vector;
use pgvector::VectorExpressionMethods;
use std::collections::HashMap;
use uuid::Uuid;

use super::Article;

impl Article {
    pub async fn find_relevant_articles_by_collection_ids(
        query_embedding: &Vector,
        conn: &mut PgConnection,
        collection_ids: &Vec<Uuid>,
    ) -> Result<Vec<(Article, f64)>, Box<dyn std::error::Error + Send + Sync>> {
        use crate::schema::articles;
        use diesel::prelude::*;

        info!("Finding relevant articles based on query embedding and collection IDs");

        let article_table = articles::table;

        let paragraph_description_results: Vec<(Article, Option<f64>)> = article_table
            .select((
                articles::all_columns,
                articles::paragraph_description_embedding
                    .cosine_distance(query_embedding)
                    .nullable(),
            ))
            .filter(articles::collection_id.eq_any(collection_ids))
            .filter(articles::paragraph_description_embedding.is_not_null())
            .order(articles::paragraph_description_embedding.cosine_distance(query_embedding))
            .limit(3)
            .load::<(Article, Option<f64>)>(conn)?;

        let bullet_points_results: Vec<(Article, Option<f64>)> = article_table
            .select((
                articles::all_columns,
                articles::bullet_points_embedding
                    .cosine_distance(query_embedding)
                    .nullable(),
            ))
            .filter(articles::collection_id.eq_any(collection_ids))
            .filter(articles::bullet_points_embedding.is_not_null())
            .order(articles::bullet_points_embedding.cosine_distance(query_embedding))
            .limit(3)
            .load::<(Article, Option<f64>)>(conn)?;

        let keywords_results: Vec<(Article, Option<f64>)> = article_table
            .select((
                articles::all_columns,
                articles::keywords_embedding
                    .cosine_distance(query_embedding)
                    .nullable(),
            ))
            .filter(articles::collection_id.eq_any(collection_ids))
            .filter(articles::keywords_embedding.is_not_null())
            .order(articles::keywords_embedding.cosine_distance(query_embedding))
            .limit(3)
            .load::<(Article, Option<f64>)>(conn)?;

        // Combine all results
        combine_and_deduplicate_results(vec![
            paragraph_description_results,
            bullet_points_results,
            keywords_results,
        ])
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
        ids: Option<&[Uuid]>,
    ) -> Result<Vec<Article>, diesel::result::Error> {
        use crate::schema::articles::dsl::*;

        info!("Performing keyword search for query: {}", query);
        let words: Vec<String> = query.split_whitespace().map(|w| w.to_lowercase()).collect();

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

        // Add filter for specific IDs if provided
        if let Some(article_ids) = ids {
            query = query.filter(id.eq_any(article_ids));
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
            .then_order_by(updated_at.desc());

        // Execute the query
        let results = query.limit(10).load::<Article>(conn)?;
        info!("Keyword search found {} results", results.len());
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

fn combine_and_deduplicate_results(
    results: Vec<Vec<(Article, Option<f64>)>>,
) -> Result<Vec<(Article, f64)>, Box<dyn std::error::Error + Send + Sync>> {
    let mut combined_results: Vec<(Article, f64)> = results
        .into_iter()
        .flatten()
        .filter_map(|(article, distance)| distance.map(|d| (article, d)))
        .collect();

    // Sort by distance (lower is better)
    combined_results.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

    // Take the top 5 unique results
    let mut unique_results = Vec::new();
    let mut seen_ids = std::collections::HashSet::new();

    for (article, distance) in combined_results {
        if seen_ids.insert(article.id) {
            unique_results.push((article, distance));
            if unique_results.len() == 5 {
                break;
            }
        }
    }

    Ok(unique_results)
}
