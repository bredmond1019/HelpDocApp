// File: src/data_processing/mod.rs

pub mod converter;
pub mod data_processor;
pub mod fetcher;
pub mod generate_embedding;

pub use converter::html_to_markdown;
pub use generate_embedding::generate_embeddings;
