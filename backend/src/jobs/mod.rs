use crate::data_processing::data_processor::SyncProcessor;
use crate::models::{article::ArticleRef, Collection};
use anyhow::Result;
use chrono::Utc;
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use tokio::sync::Mutex as TokioMutex;
use tokio::time::{sleep, Duration};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JobStatus {
    Queued,
    Running,
    Completed,
    Failed(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobInfo {
    pub id: String,
    pub status: JobStatus,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
}

pub struct JobQueue {
    sender: mpsc::Sender<Job>,
    job_statuses: Arc<Mutex<Vec<JobInfo>>>,
    num_workers: usize,
    rate_limit: Duration,
}

enum Job {
    SyncCollection(Collection),
    SyncArticle(ArticleRef, Collection),
}

impl JobQueue {
    pub fn new(
        sync_processor: Arc<SyncProcessor>,
        num_workers: usize,
        rate_limit: Duration,
    ) -> Self {
        let (sender, receiver) = mpsc::channel(32);
        let job_statuses = Arc::new(Mutex::new(Vec::new()));
        let job_queue = Self {
            sender,
            job_statuses,
            num_workers,
            rate_limit,
        };

        job_queue.spawn_workers(receiver, sync_processor);

        job_queue
    }

    fn spawn_workers(&self, receiver: mpsc::Receiver<Job>, sync_processor: Arc<SyncProcessor>) {
        let job_statuses = self.job_statuses.clone();
        let rate_limit = self.rate_limit;

        // Create a single shared receiver
        let shared_receiver = Arc::new(TokioMutex::new(receiver));

        for _ in 0..self.num_workers {
            let job_statuses = job_statuses.clone();
            let sync_processor = sync_processor.clone();
            let shared_receiver = shared_receiver.clone();

            tokio::spawn(async move {
                loop {
                    let job = shared_receiver.lock().await.recv().await;

                    match job {
                        Some(job) => {
                            let (job_id, sync_result) = match job {
                                Job::SyncCollection(collection) => {
                                    let job_id = collection.id.to_string();
                                    (job_id, sync_processor.sync_collection(&collection).await)
                                }
                                Job::SyncArticle(article_ref, collection) => {
                                    let job_id = article_ref.id.to_string();
                                    (
                                        job_id,
                                        sync_processor
                                            .sync_article(&article_ref, &collection)
                                            .await,
                                    )
                                }
                            };

                            Self::update_job_status(&job_statuses, &job_id, JobStatus::Running);
                            info!("Starting sync job: {}", job_id);

                            match sync_result {
                                Ok(_) => {
                                    info!("Sync job completed successfully: {}", job_id);
                                    Self::update_job_status(
                                        &job_statuses,
                                        &job_id,
                                        JobStatus::Completed,
                                    );
                                }
                                Err(e) => {
                                    let error_msg = format!("Sync job failed: {}", e);
                                    error!("{}", error_msg);
                                    Self::update_job_status(
                                        &job_statuses,
                                        &job_id,
                                        JobStatus::Failed(error_msg),
                                    );
                                }
                            }
                        }
                        None => break, // Channel closed, exit the loop
                    }
                    sleep(rate_limit).await;
                }
            });
        }
    }

    async fn enqueue_job<T>(&self, job: Job, id: String) -> Result<String> {
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

    pub async fn enqueue_sync_collection_job(&self, collection: Collection) -> Result<String> {
        self.enqueue_job::<Collection>(
            Job::SyncCollection(collection.clone()),
            collection.id.to_string(),
        )
        .await
    }

    pub async fn enqueue_sync_article_job(
        &self,
        article_ref: ArticleRef,
        collection: Collection,
    ) -> Result<String> {
        self.enqueue_job::<ArticleRef>(
            Job::SyncArticle(article_ref.clone(), collection),
            article_ref.id.to_string(),
        )
        .await
    }

    pub fn get_job_status(&self, job_id: &str) -> Option<JobStatus> {
        self.job_statuses
            .lock()
            .expect("Failed to lock job statuses")
            .iter()
            .find(|job| job.id == job_id)
            .map(|job| job.status.clone())
    }

    fn update_job_status(job_statuses: &Arc<Mutex<Vec<JobInfo>>>, job_id: &str, status: JobStatus) {
        let mut statuses = job_statuses.lock().expect("Failed to lock job statuses");
        if let Some(job) = statuses.iter_mut().find(|job| job.id == job_id) {
            job.status = status;
            job.updated_at = Utc::now();
        }
    }
}
