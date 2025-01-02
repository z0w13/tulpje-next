use std::collections::HashMap;

use bb8_redis::RedisConnectionManager;

use tulpje_framework::context;
use twilight_model::application::command::Command;

#[derive(Clone)]
pub struct Services {
    pub redis: bb8::Pool<RedisConnectionManager>,
    pub db: sqlx::PgPool,
    pub guild_commands: HashMap<String, Vec<Command>>,
}

pub type Context = context::Context<Services>;
pub type ComponentInteractionContext = context::ComponentInteractionContext<Services>;
pub type CommandContext = context::CommandContext<Services>;
pub type EventContext = context::EventContext<Services>;
pub type TaskContext = context::TaskContext<Services>;
