// File: src/data_processing/mod.rs

pub mod converter;
pub mod embedding;
pub mod fetcher;
pub mod sync_processor;

pub use converter::html_to_markdown;
pub use embedding::generate_embeddings;
