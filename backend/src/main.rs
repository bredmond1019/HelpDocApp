use actix_cors::Cors;
use actix_web::middleware::Logger;
use actix_web::{web, App, HttpServer};
use dotenv::dotenv;
use std::{env, sync::Arc};

use backend::db::DbPool;
use backend::jobs::JobQueue;
use backend::routes;
use backend::{data_processing::data_processor::SyncProcessor, db};

use log::{error, info};
use log4rs;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load environment variables
    dotenv().ok();

    // Initialize logger
    match log4rs::init_file("log4rs.yaml", Default::default()) {
        Ok(_) => info!("Logger initialized successfully"),
        Err(e) => error!("Failed to initialize logger: {}", e),
    }

    info!("Starting application");
    info!("API_KEY: {:?}", env::var("API_KEY"));
    info!("DATABASE_URL: {:?}", env::var("DATABASE_URL"));
    info!("QDRANT_URL: {:?}", env::var("QDRANT_URL"));
    info!("RUST_LOG: {:?}", env::var("RUST_LOG"));
    info!("RUST_BACKTRACE: {:?}", env::var("RUST_BACKTRACE"));

    let pool: DbPool = db::init_pool();

    let sync_processor = Arc::new(
        SyncProcessor::new(Arc::new(pool.clone()))
            .await
            .expect("Failed to create SyncProcessor"),
    );

    let job_queue = Arc::new(JobQueue::new(sync_processor.clone()));

    // Start the server
    info!("Server listening on 127.0.0.1:3000");
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(sync_processor.clone()))
            .app_data(web::Data::new(job_queue.clone()))
            .wrap(Logger::default())
            .wrap(Cors::permissive())
            .configure(routes::init_routes)
    })
    .bind("127.0.0.1:3000")?
    .run()
    .await
}
