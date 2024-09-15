use actix_web::{post, web, HttpResponse, Responder};
use std::sync::Arc;

use crate::services::search_service::{ArticleResult, SearchQuery, SearchResult, SearchService};
use log::info;

#[post("/search")]
async fn search(
    query: web::Json<SearchQuery>,
    search_service: web::Data<Arc<SearchService>>,
) -> impl Responder {
    info!("Received search request with query: {}", query.query);
    let original_query = query.query.clone();

    // Step 1: Expand the query
    info!("Expanding query");
    let expanded_query = match search_service.expand_query(&original_query).await {
        Ok(expanded) => {
            info!("Query expanded to: {}", expanded);
            expanded
        },
        Err(e) => {
            log::error!("Failed to expand query: {}", e);
            info!("Using original query due to expansion failure");
            original_query.clone() // Use original query if expansion fails
        }
    };

    // Step 2: Perform two-stage retrieval
    info!("Performing two-stage retrieval");
    match search_service.two_stage_retrieval(&expanded_query).await {
        Ok(article_contents) => {
            info!("Two-stage retrieval successful, found {} results", article_contents.len());
            // Step 3: Convert the results to the desired format
            let articles: Vec<ArticleResult> = article_contents
                .into_iter()
                .enumerate()
                .map(|(_, article)| ArticleResult {
                    id: article.id,
                    title: article.title,
                    content: article.markdown_content.unwrap_or("No content found".to_string()),
                    slug: article.slug,
                })
                .collect();

            let result = SearchResult {
                articles,
                expanded_query,
            };

            HttpResponse::Ok().json(result)
        }
        Err(e) => {
            log::error!("Search failed: {}", e);
            info!("Returning internal server error due to search failure");
            HttpResponse::InternalServerError().body(format!("Search failed: {}", e))
        }
    }
}