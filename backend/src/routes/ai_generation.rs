use std::sync::Arc;

use actix_web::{get, web, HttpResponse, Responder};

use crate::{db::DbPool, models::Article, services::AIService};

#[get("/generate-metadata")]
pub async fn generate_metadata(
    ai_service: web::Data<AIService>,
    pool: web::Data<Arc<DbPool>>,
) -> impl Responder {
    let mut conn = pool.get().expect("couldn't get db connection from pool");
    let article = Article::load_all(&mut conn).expect("couldn't get article")[0].clone();

    let response = ai_service.generate_article_metadata(&article).await;

    match response {
        Ok(metadata) => HttpResponse::Ok().json(metadata),
        Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
    }
}
