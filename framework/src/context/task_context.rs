use std::sync::Arc;
use twilight_http::Client;
use twilight_model::id::{marker::ApplicationMarker, Id};

use super::Context;

#[derive(Debug)]
pub struct TaskContext<T: Clone + Send + Sync> {
    pub application_id: Id<ApplicationMarker>,
    pub services: T,
    pub client: Arc<Client>,
}

impl<T: Clone + Send + Sync> TaskContext<T> {
    pub fn from_context(ctx: Context<T>) -> Self {
        Self {
            application_id: ctx.application_id,
            services: ctx.services,
            client: ctx.client,
        }
    }
}
