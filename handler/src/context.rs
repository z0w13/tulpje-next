use bb8_redis::RedisConnectionManager;

use tulpje_framework::context;

#[derive(Clone, Debug)]
pub struct Services {
    pub redis: bb8::Pool<RedisConnectionManager>,
    pub db: sqlx::PgPool,
}

pub type Context = context::Context<Services>;
pub type ComponentInteractionContext = context::ComponentInteractionContext<Services>;
pub type CommandContext = context::CommandContext<Services>;
pub type EventContext = context::EventContext<Services>;
