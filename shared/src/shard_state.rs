use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct ShardState {
    pub shard_id: u64,

    pub up: bool,
    pub disconnect_count: u64,

    pub latency: u64,

    pub last_started: u64,
    pub last_heartbeat: u64,
    pub last_connection: u64,
}

impl ShardState {
    pub fn new(shard_id: u64) -> Self {
        Self {
            shard_id,
            last_started: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("time went backwards")
                .as_secs(),

            ..Default::default()
        }
    }
}
