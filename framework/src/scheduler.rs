use std::{collections::HashMap, future::Future, pin::Pin};

use async_cron_scheduler::{Job, JobId, Scheduler as CronScheduler};
use chrono::Utc;

use crate::{
    context::{Context, TaskContext},
    handler::task_handler::TaskHandler,
};

pub struct Scheduler {
    job_map: HashMap<String, JobId>,
    scheduler: CronScheduler<Utc>,
    runner: Option<Pin<Box<dyn Future<Output = ()> + Send + Sync>>>,
}

impl Scheduler {
    #[expect(
        clippy::new_without_default,
        reason = "we might have constructor arguments in the future, having a Default implementation feels incorrect"
    )]
    pub fn new() -> Self {
        let (scheduler, service) = CronScheduler::<Utc>::launch(tokio::time::sleep);

        Self {
            job_map: HashMap::new(),
            scheduler,
            runner: Some(Box::pin(service)),
        }
    }

    pub async fn enable_task<T: Clone + Send + Sync + 'static>(&mut self, ctx: Context<T>, task: TaskHandler<T>) {
        let job = Job::<Utc>::cron_schedule(task.cron.clone());
        let job_name = task.name.clone();
        let job_id = self
            .scheduler
            .insert(job, move |_id| {
                let job_ctx = ctx.clone();
                let job_handler = task.clone();

                tokio::spawn(async move {
                    if let Err(err) = job_handler.run(TaskContext::from_context(job_ctx)).await
                    {
                        tracing::error!("error running task {}: {}", job_handler.name, err);
                    };
                });
            })
            .await;
        self.job_map.insert(job_name, job_id);
    }

    pub async fn disable_task(&mut self, name: &str) {
        let Some(job_id) = self.job_map.get(name) else {
            return;
        };

        self.scheduler.remove(*job_id).await;
    }

    pub async fn run<T: Clone + Send + Sync + 'static>(
        &mut self,
        ctx: Context<T>,
        tasks: Vec<&TaskHandler<T>>,
    ) -> tokio::task::JoinHandle<()> {
        if self.runner.is_none() {
            panic!("Scheduler::run called twice, scheduler is already running");
        }

        for task in tasks {
            self.enable_task(ctx.clone(), task.clone()).await;
        }

        tokio::spawn(self.runner.take().unwrap())
    }
}
