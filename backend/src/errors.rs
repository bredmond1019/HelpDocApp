use std::error::Error as StdError;
use std::fmt;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum SyncError {
    #[error("API client error: {0}")]
    ApiClientError(anyhow::Error),
    #[error("Failed to fetch collections: {0}")]
    CollectionFetchError(anyhow::Error),
    #[error("Failed to fetch articles for collection {collection_id}: {error}")]
    ArticleFetchError {
        collection_id: String,
        error: anyhow::Error,
    },
    #[error("Failed to enqueue job: {0}")]
    JobEnqueueError(anyhow::Error),
    #[error("Failed to prepare jobs for collection {collection_id}: {error}")]
    JobPreparationError {
        collection_id: String,
        error: anyhow::Error,
    },
    #[error("Failed to generate or store embedding: {0}")]
    EmbeddingError(anyhow::Error),

    #[error("Other error: {0}")]
    Other(#[from] anyhow::Error),
}

#[derive(Debug)]
pub struct MetadataGenerationError(pub Box<dyn std::error::Error + Send + Sync>);

impl fmt::Display for MetadataGenerationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl StdError for MetadataGenerationError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        Some(&*self.0)
    }
}

impl From<Box<dyn std::error::Error + Send + Sync>> for MetadataGenerationError {
    fn from(err: Box<dyn std::error::Error + Send + Sync>) -> Self {
        MetadataGenerationError(err)
    }
}

#[derive(Debug)]
pub struct MetaDataError(pub anyhow::Error);

impl fmt::Display for MetaDataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl StdError for MetaDataError {}

impl From<anyhow::Error> for MetaDataError {
    fn from(err: anyhow::Error) -> Self {
        MetaDataError(err)
    }
}
