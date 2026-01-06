use crate::repository::token_repository;
use sqlx::PgPool;
use tokio_cron_scheduler::{Job, JobScheduler, JobSchedulerError};
use tracing::{error, info};

pub async fn init_scheduler(pool: PgPool) -> Result<JobScheduler, JobSchedulerError> {
    let sched = JobScheduler::new().await?;

    // Schedule: 2:00 AM daily
    // Cron format: sec min hour day_of_month month day_of_week year
    let job = Job::new_async("0 0 2 * * * *", move |_uuid, _l| {
        let pool = pool.clone();
        Box::pin(async move {
            info!("Starting scheduled token cleanup...");
            match token_repository::delete_expired_tokens(&pool).await {
                Ok(count) => info!("Deleted {} expired refresh tokens.", count),
                Err(e) => error!("Failed to delete expired tokens: {}", e),
            }
        })
    })?;

    sched.add(job).await?;
    Ok(sched)
}
