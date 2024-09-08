use crate::data_processing::data_processor::DataProcessor;
use crate::models::{article::ArticleRef, Collection};
use anyhow::Result;
use chrono::Utc;
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::env;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use tokio::sync::Mutex as TokioMutex;
use tokio::time::{sleep, Duration};

pub mod enqueue;
pub mod worker;

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

pub enum Job {
    SyncCollection(Collection),
    SyncArticle(ArticleRef, Collection),
}

impl Job {
    async fn process(
        &self,
        processor: &DataProcessor,
    ) -> Result<(String, Result<(), anyhow::Error>), anyhow::Error> {
        match self {
            Job::SyncCollection(collection) => {
                info!("Starting sync job: {}", collection.id);
                let job_id = collection.id.to_string();
                let result = processor.sync_collection(&collection).await;
                Ok((job_id, result))
            }
            Job::SyncArticle(article_ref, collection) => {
                let job_id = article_ref.id.to_string();
                let result = processor.sync_article(article_ref, collection).await;
                Ok((job_id, result))
            }
        }
    }
}

impl JobQueue {
    pub fn new(sync_processor: Arc<DataProcessor>) -> Self {
        let num_workers = env::var("JOB_QUEUE_WORKERS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(4);

        let rate_limit = env::var("JOB_QUEUE_RATE_LIMIT_MS")
            .ok()
            .and_then(|v| v.parse().ok())
            .map(Duration::from_millis)
            .unwrap_or(Duration::from_millis(500));

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
