use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct DiscordEvent {
    pub meta: DiscordEventMeta,
    pub payload: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DiscordEventMeta {
    pub uuid: uuid::Uuid, // used for tracing
    pub shard: u64,
}

impl DiscordEvent {
    pub fn new(shard: u64, payload: String) -> Self {
        Self {
            meta: DiscordEventMeta {
                uuid: uuid::Uuid::now_v7(),
                shard,
            },
            payload,
        }
    }
}
