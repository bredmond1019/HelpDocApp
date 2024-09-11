use actix_cors::Cors;
use actix_web::middleware::Logger;
use actix_web::{web, App, HttpServer};
use dotenv::dotenv;
use std::env;
use std::sync::Arc;
// use std::process::Command;

use backend::db::DbPool;
use backend::routes;
use backend::{data_processor::DataProcessor, db};

use log::{error, info};
use log4rs;

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

    let data_processor = Arc::new(
        DataProcessor::new(Arc::new(pool.clone()))
            .await
            .expect("Failed to create DataProcessor"),
    );

    // Start the server
    info!("Server listening on 127.0.0.1:3000");
    let server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(data_processor.clone()))
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
