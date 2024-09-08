use crate::data_processing::sync_processor::SyncProcessor;
use anyhow::Result;
use chrono::Utc;
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

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
}

enum Job {
    Sync(String), // String is the job ID
}

impl JobQueue {
    pub fn new(sync_processor: Arc<SyncProcessor>) -> Self {
        let (sender, mut receiver) = mpsc::channel(32);
        let job_statuses = Arc::new(Mutex::new(Vec::new()));
        let job_statuses_clone = job_statuses.clone();

        tokio::spawn(async move {
            while let Some(job) = receiver.recv().await {
                match job {
                    Job::Sync(job_id) => {
                        Self::update_job_status(&job_statuses_clone, &job_id, JobStatus::Running);
                        info!("Starting sync job: {}", job_id);
                        match sync_processor.sync_all().await {
                            Ok(_) => {
                                info!("Sync job completed successfully: {}", job_id);
                                Self::update_job_status(
                                    &job_statuses_clone,
                                    &job_id,
                                    JobStatus::Completed,
                                );
                            }
                            Err(e) => {
                                let error_msg = format!("Sync job failed: {}", e);
                                error!("{}", error_msg);
                                Self::update_job_status(
                                    &job_statuses_clone,
                                    &job_id,
                                    JobStatus::Failed(error_msg),
                                );
                            }
                        }
                    }
                }
            }
        });

        Self {
            sender,
            job_statuses,
        }
    }

    pub async fn enqueue_sync_job(&self) -> Result<String> {
        let job_id = uuid::Uuid::new_v4().to_string();
        let job_info = JobInfo {
            id: job_id.clone(),
            status: JobStatus::Queued,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        self.job_statuses.lock().unwrap().push(job_info);

        self.sender
            .send(Job::Sync(job_id.clone()))
            .await
            .map_err(|e| anyhow::anyhow!("Failed to enqueue sync job: {}", e))?;

        Ok(job_id)
    }

    pub fn get_job_status(&self, job_id: &str) -> Option<JobStatus> {
        self.job_statuses
            .lock()
            .unwrap()
            .iter()
            .find(|job| job.id == job_id)
            .map(|job| job.status.clone())
    }

    fn update_job_status(job_statuses: &Arc<Mutex<Vec<JobInfo>>>, job_id: &str, status: JobStatus) {
        let mut statuses = job_statuses.lock().unwrap();
        if let Some(job) = statuses.iter_mut().find(|job| job.id == job_id) {
            job.status = status;
            job.updated_at = Utc::now();
        }
    }
}
