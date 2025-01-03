use bb8_redis::RedisConnectionManager;

use tulpje_framework::{context, Registry};

#[derive(Clone)]
pub struct Services {
    pub redis: bb8::Pool<RedisConnectionManager>,
    pub db: sqlx::PgPool,
    pub registry: Registry<Services>,
}

pub type Context = context::Context<Services>;
pub type ComponentInteractionContext = context::ComponentInteractionContext<Services>;
pub type CommandContext = context::CommandContext<Services>;
pub type EventContext = context::EventContext<Services>;
pub type TaskContext = context::TaskContext<Services>;
