use chrono::Utc;
use uuid::Uuid;

use super::{Job, JobInfo, JobQueue, JobStatus};

impl JobQueue {
    pub async fn enqueue_job(&self, job: Job) -> Result<Uuid, anyhow::Error> {
        let id = Uuid::new_v4();
        let job_info = JobInfo {
            id,
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
}
