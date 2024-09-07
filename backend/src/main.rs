use actix_web::{App, HttpServer};
use dotenv::dotenv;
use std::{env, sync::Arc};

use backend::db::DbPool;
use backend::routes;
use backend::{data_processing::sync_processor::SyncProcessor, db};
use routes::sync_route;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load environment variables
    dotenv().ok();
    env::set_var("RUST_BACKTRACE", "1");

    let pool: DbPool = db::init_pool();

    // Create the sync processor
    let sync_processor = Arc::new(
        SyncProcessor::new(Arc::new(pool.clone()))
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
