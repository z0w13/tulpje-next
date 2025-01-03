use std::{future::Future, pin::Pin};

use async_cron_scheduler::cron::Schedule;
use chrono::{DateTime, Utc};

use crate::context::TaskContext;
use crate::Error;

pub(crate) type TaskFunc<T> =
    fn(TaskContext<T>) -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send>>;

#[derive(Clone)]
pub struct TaskHandler<T: Clone> {
    pub module: String,
    pub name: String,
    pub cron: Schedule,
    pub func: TaskFunc<T>,
}

impl<T: Clone> TaskHandler<T> {
    pub async fn run(&self, ctx: TaskContext<T>) -> Result<(), Error> {
        // can add more handling/parsing/etc here in the future
        (self.func)(ctx).await
    }

    pub fn next_run(&self) -> Option<DateTime<Utc>> {
        self.cron.upcoming(Utc).next()
    }
}
