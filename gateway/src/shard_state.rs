use std::time::{SystemTime, UNIX_EPOCH};

use bb8_redis::{redis::AsyncCommands, RedisConnectionManager};
use serde::{Deserialize, Serialize};
use twilight_gateway::{Event, Latency};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct ShardState {
    pub shard_id: u64,

    pub up: bool,
    pub disconnect_count: u64,

    pub latency: u64,

    pub last_heartbeat: u64,
    pub last_connection: u64,
}

impl ShardState {
    pub fn new(shard_id: u64) -> Self {
        Self {
            shard_id,

            ..Default::default()
        }
    }
}

pub struct ShardManager {
    pub redis: bb8::Pool<RedisConnectionManager>,
    pub shard: ShardState,
}

impl ShardManager {
    pub fn new(redis: bb8::Pool<RedisConnectionManager>, shard_id: u64) -> Self {
        Self {
            redis,
            shard: ShardState::new(shard_id),
        }
    }

    pub async fn handle_event(
        &mut self,
        event: Event,
        latency: &Latency,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match event {
            Event::Ready(_) => self.ready_or_resumed(false).await,
            Event::Resumed => self.ready_or_resumed(true).await,
            Event::GatewayClose(_) => self.socket_closed().await,
            Event::GatewayHeartbeatAck => self.heartbeated(latency).await,
            _ => Ok(()),
        }
    }

    async fn save_shard(&self) -> Result<(), Box<dyn std::error::Error>> {
        let json_shard = serde_json::to_string(&self.shard)?;

        self.redis
            .get()
            .await?
            .hset::<&str, String, String, ()>(
                "tulpje:shard_status",
                self.shard.shard_id.to_string(),
                json_shard,
            )
            .await
            .map_err(|err| err.into())
    }

    async fn ready_or_resumed(&mut self, resumed: bool) -> Result<(), Box<dyn std::error::Error>> {
        tracing::info!(
            "shard {} {}",
            self.shard.shard_id,
            if resumed { "resumed" } else { "ready" }
        );

        self.shard.up = true;
        self.shard.last_connection = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time went backwards")
            .as_secs();

        self.save_shard().await
    }

    async fn socket_closed(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        tracing::info!("shard {} closed", self.shard.shard_id);

        self.shard.up = false;
        self.shard.disconnect_count += 1;

        self.save_shard().await
    }

    async fn heartbeated(&mut self, latency: &Latency) -> Result<(), Box<dyn std::error::Error>> {
        self.shard.up = true;
        self.shard.last_heartbeat = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time went backwards")
            .as_secs();

        self.shard.latency = latency
            .recent()
            .first()
            .expect("no latency measurement after heartbeat")
            .as_millis()
            .try_into()
            .expect("couldn't convert into u64");

        self.save_shard().await
    }
}
