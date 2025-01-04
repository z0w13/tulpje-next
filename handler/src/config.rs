use serde::{Deserialize, Serialize};
use serde_envfile::Error;

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub discord_proxy: String,
    pub rabbitmq_address: String,
    pub redis_url: String,
    pub database_url: String,

    pub handler_id: u32,
    pub handler_count: u32,
}

impl Config {
    pub fn from_env() -> Result<Self, Error> {
        serde_envfile::from_env()
    }
}
