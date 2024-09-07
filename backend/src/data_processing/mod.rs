// File: src/data_processing/mod.rs

pub mod converter;
pub mod fetcher;
pub mod generate_embedding;
pub mod sync_processor;

pub use converter::html_to_markdown;
pub use generate_embedding::generate_embeddings;
