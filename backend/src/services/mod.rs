pub mod ai_service;
pub mod embedding_service;
pub mod chat_server;
pub mod chat_session;

pub use ai_service::AIModel;
pub use embedding_service::generate_and_store_embedding;