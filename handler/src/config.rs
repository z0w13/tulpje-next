use serde::{Deserialize, Serialize};
use serde_envfile::Error;

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub discord_proxy: String,
    pub rabbitmq_address: String,
    pub redis_url: String,
}

impl Config {
    pub fn from_env() -> Result<Self, Error> {
        serde_envfile::from_env()
    }
}
