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

mod enqueue;
mod spawn;

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
