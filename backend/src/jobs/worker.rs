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
                            let (job_id, sync_result) = match job {
                                Job::SyncCollection(collection) => {
                                    let job_id = collection.id.to_string();
                                    (job_id, data_processor.sync_collection(&collection).await)
                                }
                                Job::SyncArticle(article_ref, collection) => {
                                    let job_id = article_ref.id.to_string();
                                    (
                                        job_id,
                                        data_processor
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

    fn spawn_workers(&self, mut receiver: mpsc::Receiver<Job>, data_processor: Arc<DataProcessor>) {
        let job_statuses = Arc::clone(&self.job_statuses);
        let rate_limit = self.rate_limit;

        for _ in 0..self.num_workers {
            let job_statuses = Arc::clone(&job_statuses);
            let data_processor = Arc::clone(&data_processor);

            tokio::spawn(async move {
                while let Some(job) = receiver.recv().await {
                    // Process the job based on its type
                    let result = job.process(&data_processor).await;

                    // Update job status based on the result
                    let status = match result {
                        Ok(_) => JobStatus::Completed,
                        Err(e) => JobStatus::Failed(e.to_string()),
                    };

                    Self::update_job_status(&job_statuses, &job_id, JobStatus::Running);
                    info!("Starting sync job: {}", job_id);

                    match sync_result {
                        Ok(_) => {
                            info!("Sync job completed successfully: {}", job_id);
                            Self::update_job_status(&job_statuses, &job_id, JobStatus::Completed);
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

                    // Apply rate limiting
                    sleep(rate_limit).await;
                }
            });
        }
    }
}
