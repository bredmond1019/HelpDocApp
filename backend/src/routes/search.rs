use actix_web::{post, web, HttpResponse, Responder};
use std::sync::Arc;

use crate::services::search_service::{ArticleResult, SearchQuery, SearchResult, SearchService};



#[post("/search")]
async fn search(
    query: web::Json<SearchQuery>,
    search_service: web::Data<Arc<SearchService>>,
) -> impl Responder {
    let original_query = query.query.clone();

    // Step 1: Expand the query
    let expanded_query = match search_service.expand_query(&original_query).await {
        Ok(expanded) => expanded,
        Err(e) => {
            log::error!("Failed to expand query: {}", e);
            original_query.clone() // Use original query if expansion fails
        }
    };

    // Step 2: Perform two-stage retrieval
    match search_service.two_stage_retrieval(&expanded_query).await {
        Ok(article_contents) => {
            // Step 3: Convert the results to the desired format
            let articles: Vec<ArticleResult> = article_contents
                .into_iter()
                .enumerate()
                .map(|(index, content)| ArticleResult {
                    id: uuid::Uuid::new_v4(), // You might want to return actual article IDs
                    title: format!("Result {}", index + 1), // You might want to return actual titles
                    content,
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
            HttpResponse::InternalServerError().body(format!("Search failed: {}", e))
        }
    }
}