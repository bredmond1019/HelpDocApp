use chrono::Utc;

use crate::models::{article::ArticleRef, Collection};

use super::{Job, JobInfo, JobQueue, JobStatus};

impl JobQueue {
    async fn enqueue_job<T>(&self, job: Job, id: String) -> Result<String, anyhow::Error> {
        let job_info = JobInfo {
            id: id.clone(),
            status: JobStatus::Queued,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        {
            let mut statuses = self
                .job_statuses
                .lock()
                .expect("Failed to lock job statuses");
            statuses.push(job_info);
        }

        self.sender
            .send(job)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to enqueue job: {}", e))?;

        Ok(id)
    }

    pub async fn enqueue_sync_collection_job(
        &self,
        collection: Collection,
    ) -> Result<String, anyhow::Error> {
        self.enqueue_job(
            Job::SyncCollection(collection.clone()),
            collection.id.to_string(),
        )
        .await
    }

    pub async fn enqueue_sync_article_job(
        &self,
        article_ref: ArticleRef,
        collection: Collection,
    ) -> Result<String, anyhow::Error> {
        self.enqueue_job(
            Job::SyncArticle(article_ref.clone(), collection),
            article_ref.id.to_string(),
        )
        .await
    }
}
