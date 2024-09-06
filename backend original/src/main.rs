pub mod data_processing;
mod db;
pub mod models;
mod routes;

use crate::data_processing::sync_processor::SyncProcessor;

use actix_web::{App, HttpServer};
use dotenv::dotenv;
use routes::sync_route;
use sqlx::postgres::PgPoolOptions;
use std::{env, sync::Arc};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load environment variables
    dotenv().ok();
    env::set_var("RUST_BACKTRACE", "1");

    // Initialize PostgreSQL connection
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = Arc::new(
        PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("Failed to connect to database"),
    );

    // Create the sync processor
    let sync_processor = Arc::new(
        SyncProcessor::new(pool.clone())
            .await
            .expect("Failed to create SyncProcessor"),
    );

    // Start the server
    println!("Server listening on 127.0.0.1:3000");
    HttpServer::new(move || {
        App::new()
            .app_data(sync_processor.clone())
            .service(sync_route())
    })
    .bind("127.0.0.1:3000")?
    .run()
    .await
}
