use std::collections::HashMap;

use async_cron_scheduler::{Job, JobId, Scheduler as CronScheduler};
use chrono::Utc;

use crate::{
    context::{Context, TaskContext},
    handler::task_handler::TaskHandler,
};

pub struct Scheduler {
    job_map: HashMap<String, JobId>,
    scheduler: Option<CronScheduler<Utc>>,
}

impl Scheduler {
    #[expect(
        clippy::new_without_default,
        reason = "we might have constructor arguments in the future, having a Default implementation feels incorrect"
    )]
    pub fn new() -> Self {
        Self {
            job_map: HashMap::new(),
            scheduler: None,
        }
    }

    pub async fn run<T: Clone + Send + Sync + 'static>(
        &mut self,
        ctx: Context<T>,
        tasks: Vec<&TaskHandler<T>>,
    ) -> tokio::task::JoinHandle<()> {
        let (mut scheduler, sched_service) = CronScheduler::<Utc>::launch(tokio::time::sleep);

        for task in tasks {
            let loop_ctx = ctx.clone();
            let loop_handler = task.clone();

            let job = Job::<Utc>::cron_schedule(task.cron.clone());
            let job_id = scheduler
                .insert(job, move |_id| {
                    let job_ctx = loop_ctx.clone();
                    let job_handler = loop_handler.clone();

                    tokio::spawn(async move {
                        if let Err(err) = job_handler.run(TaskContext::from_context(job_ctx)).await
                        {
                            tracing::error!("error running task {}: {}", job_handler.name, err);
                        };
                    });
                })
                .await;
            self.job_map.insert(task.name.clone(), job_id);
        }

        self.scheduler = Some(scheduler);
        tokio::spawn(sched_service)
    }
}
