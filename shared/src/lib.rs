use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct DiscordEvent {
    pub uuid: uuid::Uuid, // used for tracing
    pub payload: String,
}

impl DiscordEvent {
    pub fn new(payload: String) -> Self {
        Self {
            uuid: uuid::Uuid::now_v7(),
            payload,
        }
    }
}
