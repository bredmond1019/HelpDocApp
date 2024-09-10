use crate::data_processor::DataProcessor;
use crate::models::{JobInfo, JobStatus};

use chrono::Utc;
use std::env;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use tokio::time::Duration;
use uuid::Uuid;

use log::info;

use super::Job;

#[derive(Clone)]
pub struct JobQueue {
    pub sender: mpsc::Sender<Job>,
    pub job_statuses: Arc<Mutex<Vec<JobInfo>>>,
    pub num_workers: usize,
    pub rate_limit: Duration,
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

        info!("Job queue initialized with {} workers", num_workers);

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

    pub fn update_job_status(
        job_statuses: &Arc<Mutex<Vec<JobInfo>>>,
        job_id: Uuid,
        status: JobStatus,
    ) {
        let mut statuses = job_statuses.lock().expect("Failed to lock job statuses");
        if let Some(job) = statuses.iter_mut().find(|job| job.id == job_id) {
            job.status = status;
            job.updated_at = Utc::now();
        }
    }

    pub fn queue_size(&self) -> usize {
        self.sender.capacity() - self.sender.max_capacity()
    }
}
