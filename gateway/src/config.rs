use serde::{Deserialize, Serialize};
use serde_envfile::Error;

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub discord_token: String,
    pub discord_proxy: String,
    pub discord_gateway_queue: String,
    pub shard_id: u64,
    pub shard_count: u64,
    pub rabbitmq_address: String,
    pub redis_url: String,
}

impl Config {
    pub fn from_env() -> Result<Self, Error> {
        serde_envfile::from_env()
    }
}
