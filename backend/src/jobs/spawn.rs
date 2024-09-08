impl JobQueue {
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
}
