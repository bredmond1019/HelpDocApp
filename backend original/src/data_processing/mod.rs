// File: src/data_processing/mod.rs

pub mod converter;
pub mod fetcher;
pub mod storage;
pub mod sync_processor;

pub use converter::html_to_markdown;
pub use storage::{generate_embeddings, store_in_postgres};
