use std::sync::Arc;

use bb8_redis::RedisConnectionManager;

use tulpje_framework::{context, Registry};

#[derive(Clone)]
pub struct Services {
    // NOTE: Internally uses an Arc, "cheap" to clone
    pub redis: bb8::Pool<RedisConnectionManager>,
    // NOTE: Internally uses an Arc, "cheap" to clone
    pub db: sqlx::PgPool,
    // NOTE: Cloning Registry would be very expensive and clones all the internal
    //       HashMaps, etc. so we should wrap it in an Arc
    pub registry: Arc<Registry<Services>>,
}

pub type Context = context::Context<Services>;
pub type ComponentInteractionContext = context::ComponentInteractionContext<Services>;
pub type CommandContext = context::CommandContext<Services>;
pub type EventContext = context::EventContext<Services>;
pub type TaskContext = context::TaskContext<Services>;
