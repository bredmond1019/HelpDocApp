use std::sync::Arc;

use actix_web::{get, web, HttpResponse, Responder};
use log::{error, info};
use serde_json::json;

use crate::{data_processor::DataProcessor, db::DbPool, models::Article, services::AIService};

#[get("/generate-metadata")]
pub async fn generate_metadata(
    ai_service: web::Data<Arc<AIService>>,
    pool: web::Data<Arc<DbPool>>,
    data_processor: web::Data<Arc<DataProcessor>>,
) -> impl Responder {
    let mut conn = pool.get().expect("couldn't get db connection from pool");
    let article = Article::load_all(&mut conn).expect("couldn't get article")[0].clone();

    let ai_generator = ai_service.clone();

    tokio::spawn(async move {
        let response = ai_generator.generate_article_metadata(&article).await;
        match response {
            Ok(metadata) => {
                info!("Metadata generated: {}", metadata);
                let (paragraph, bullets, keywords) = data_processor.parse_metadata(&metadata);

                info!("Paragraph: {}", paragraph);
                info!("Bullets: {}", bullets);
                info!("Keywords: {}", keywords);
            }
            Err(e) => error!("Error generating metadata: {}", e),
        }
    });

    HttpResponse::Accepted().json(json!({
        "message": "Generating metadata",
    }))
}
