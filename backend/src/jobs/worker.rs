use log::{error, info};
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::Mutex as TokioMutex;
use tokio::time::sleep;

use crate::{data_processing::data_processor::DataProcessor, jobs::JobStatus};

use super::{Job, JobQueue};

impl JobQueue {
    pub fn spawn_workers(&self, receiver: mpsc::Receiver<Job>, data_processor: Arc<DataProcessor>) {
        let job_statuses = self.job_statuses.clone();
        let rate_limit = self.rate_limit;

        // Create a single shared receiver
        let shared_receiver = Arc::new(TokioMutex::new(receiver));

        for _ in 0..self.num_workers {
            let job_statuses = job_statuses.clone();
            let data_processor = data_processor.clone();
            let shared_receiver = shared_receiver.clone();

            tokio::spawn(async move {
                loop {
                    let job = shared_receiver.lock().await.recv().await;

                    match job {
                        Some(job) => {
                            let (job_id, result) = job
                                .process(&data_processor)
                                .await
                                .expect("Failed to process job"); // TODO: Handle this better

                            Self::update_job_status(&job_statuses, &job_id, JobStatus::Running);
                            info!("Starting sync job: {}", job_id);

                            match result {
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
}
