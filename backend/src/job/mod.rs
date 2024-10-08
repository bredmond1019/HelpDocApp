use crate::errors::SyncError;
use crate::models::articles::Article;
use crate::models::{articles::ArticleRef, Collection};
use crate::services::DataProcessor;

use anyhow::Result;
use uuid::Uuid;

pub mod enqueue;
pub mod job_queue;
pub mod worker;

pub use job_queue::JobQueue;

#[derive(Debug, Clone)]
pub enum Job {
    SyncCollection(Collection),
    StoreCollection(Collection),
    SyncArticle(ArticleRef, Collection),
    StoreArticle(Article),
    ConvertHtmlToMarkdown(Article),
    EnqueueJobs(Vec<Job>),
}

impl Job {
    async fn process(
        &self,
        processor: &DataProcessor,
        job_queue: &JobQueue,
    ) -> Result<(Uuid, Result<(), anyhow::Error>), anyhow::Error> {
        let job_id = Uuid::new_v4();
        log::info!("Starting to process job ID: {}", job_id);

        let result = match self {
            Job::SyncCollection(collection) => {
                log::info!(
                    "Processing SyncCollection for collection: {}",
                    collection.id
                );
                let result = processor.prepare_sync_collection(collection).await;
                log::info!("SyncCollection completed for collection: {}", collection.id);
                result.map(|_| ())
            }
            Job::StoreCollection(collection) => {
                log::info!(
                    "Processing StoreCollection for collection: {}",
                    collection.id
                );
                let result = processor.sync_collection(collection).await;
                log::info!(
                    "StoreCollection completed for collection: {}",
                    collection.id
                );
                result
            }
            Job::SyncArticle(article_ref, collection) => {
                log::info!("Processing SyncArticle for article: {}", article_ref.id);
                let result = processor.sync_article(article_ref, collection).await;
                log::info!("SyncArticle completed for article: {}", article_ref.id);
                result
            }
            Job::EnqueueJobs(jobs) => {
                for job in jobs {
                    job_queue
                        .enqueue_job(job.clone())
                        .await
                        .map_err(SyncError::JobEnqueueError)?;
                }
                Ok(())
            }
            Job::StoreArticle(article) => {
                log::info!("Processing StoreArticle for article: {}", article.id);
                processor.store_article(article).await?;
                log::info!("StoreArticle completed for article: {}", article.id);
                Ok(())
            }
            Job::ConvertHtmlToMarkdown(article) => {
                log::info!(
                    "Processing ConvertHtmlToMarkdown for article: {}",
                    article.id
                );
                let result = processor.convert_html_to_markdown(article).await;
                log::info!(
                    "ConvertHtmlToMarkdown completed for article: {}",
                    article.id
                );
                result
            }
        };

        log::info!("Finished processing job: {:?} (ID: {})", self, job_id);
        Ok((job_id, result))
    }
}
