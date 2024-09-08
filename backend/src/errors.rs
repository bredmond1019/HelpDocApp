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
    #[error("Other error: {0}")]
    Other(#[from] anyhow::Error),
}

// This implementation is no longer needed as it's covered by the #[from] attributes above
// impl From<anyhow::Error> for SyncError {
//     fn from(error: anyhow::Error) -> Self {
//         SyncError::Other(error)
//     }
// }
