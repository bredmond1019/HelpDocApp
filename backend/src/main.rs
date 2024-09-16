use actix::Actor;
use actix_cors::Cors;
use actix_web::middleware::Logger;
use actix_web::{web, App, HttpServer};
use backend::services::search::SearchService;
use backend::services::{AIService, EmbeddingService};
use dotenv::dotenv;
use log::{error, info};
use log4rs;
use std::env;
use std::sync::Arc;
// use std::process::Command;

use backend::db;
use backend::db::DbPool;
use backend::routes;
use backend::services::{chat::chat_server::ChatServer, data_processor::DataProcessor};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load environment variables
    dotenv().ok();
    env::set_var("RUST_LOG", "info");
    env::set_var("RUST_BACKTRACE", "1");

    // Initialize logger
    match log4rs::init_file("log4rs.yaml", Default::default()) {
        Ok(_) => info!("Logger initialized successfully"),
        Err(e) => error!("Failed to initialize logger: {}", e),
    }

    info!("Starting application");

    // Python Service for Embedding
    // let mut python_service = Command::new("python")
    //     .arg("python_services/embedding_service.py")
    //     .spawn()
    //     .expect("Failed to start Python service");
    // info!("Python service started with PID: {:?}", python_service.id());

    let pool: DbPool = db::init_pool();
    let arc_pool = Arc::new(pool.clone());

    info!("Initializing DataProcessor");
    let data_processor = Arc::new(
        DataProcessor::new(arc_pool.clone())
            .await
            .expect("Failed to create DataProcessor"),
    );
    info!("DataProcessor initialized");

    info!("Initializing ChatServer");
    let chat_server = ChatServer::new(arc_pool.clone()).start();
    info!("ChatServer initialized and started");

    info!("Initializing EmbeddingService");
    let embedding_service = Arc::new(EmbeddingService::new());
    info!("EmbeddingService initialized");

    info!("Initializing AIService");
    let ai_service = Arc::new(AIService::new());
    info!("AIService initialized");

    info!("Initializing SearchService");
    let search_service = Arc::new(SearchService::new(arc_pool.clone(), ai_service.clone()));
    info!("SearchService initialized");

    // Start the server
    info!("Server listening on 127.0.0.1:3000");
    let server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(arc_pool.clone()))
            .app_data(web::Data::new(data_processor.clone()))
            .app_data(web::Data::new(chat_server.clone()))
            .app_data(web::Data::new(embedding_service.clone()))
            .app_data(web::Data::new(search_service.clone()))
            .app_data(web::Data::new(ai_service.clone()))
            .wrap(Logger::default())
            .wrap(Cors::permissive())
            .configure(routes::init_routes)
    })
    .bind("127.0.0.1:3000")?
    .run();

    let server_result = server.await;

    // python_service.kill().expect("Failed to stop Python service");

    server_result
}
