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
