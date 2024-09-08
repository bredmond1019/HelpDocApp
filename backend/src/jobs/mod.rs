use crate::data_processor::data_processor::DataProcessor;
use crate::models::Article;
use crate::models::{article::ArticleRef, Collection};

use anyhow::Result;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::env;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use tokio::time::Duration;
use uuid::Uuid;

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
    pub id: Uuid,
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
    StoreCollection(Collection),
    SyncArticle(ArticleRef, Collection),
    StoreArticle(Article),
    ConvertHtmlToMarkdown(Article),
    GenerateEmbeddings(Article),
}

impl Job {
    async fn process(
        &self,
        processor: &DataProcessor,
    ) -> Result<(Uuid, Result<(), anyhow::Error>), anyhow::Error> {
        match self {
            Job::SyncCollection(collection) => {
                let job_id = Uuid::new_v4();
                let result = processor.prepare_sync_collection(collection).await;
                Ok((job_id, result.map(|_| ())))
            }
            Job::StoreCollection(collection) => {
                let job_id = Uuid::new_v4();
                let result = processor.sync_collection(collection).await;
                Ok((job_id, result))
            }
            Job::SyncArticle(article_ref, collection) => {
                let job_id = Uuid::new_v4();
                let result = processor.sync_article(article_ref, collection).await;
                Ok((job_id, result.map(|_| ())))
            }
            Job::StoreArticle(article) => {
                let job_id = Uuid::new_v4();
                let result = processor.store_article(article).await;
                Ok((job_id, result))
            }
            Job::ConvertHtmlToMarkdown(article) => {
                let job_id = Uuid::new_v4();
                let result = processor.convert_html_to_markdown(article).await;
                Ok((job_id, result))
            }
            Job::GenerateEmbeddings(article) => {
                let job_id = Uuid::new_v4();
                let result = processor.generate_article_embeddings(&article).await;
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

    pub fn get_job_status(&self, job_id: Uuid) -> Option<JobStatus> {
        self.job_statuses
            .lock()
            .expect("Failed to lock job statuses")
            .iter()
            .find(|job| job.id == job_id)
            .map(|job| job.status.clone())
    }

    fn update_job_status(job_statuses: &Arc<Mutex<Vec<JobInfo>>>, job_id: Uuid, status: JobStatus) {
        let mut statuses = job_statuses.lock().expect("Failed to lock job statuses");
        if let Some(job) = statuses.iter_mut().find(|job| job.id == job_id) {
            job.status = status;
            job.updated_at = Utc::now();
        }
    }
}
