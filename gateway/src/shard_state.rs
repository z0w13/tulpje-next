use std::{
    collections::HashSet,
    time::{SystemTime, UNIX_EPOCH},
};

use bb8_redis::{redis::AsyncCommands, RedisConnectionManager};
use twilight_gateway::{Event, Latency};

use tulpje_shared::shard_state::ShardState;
use twilight_model::gateway::payload::incoming::{GuildCreate, GuildDelete, Hello, Ready};

pub struct ShardManager {
    pub redis: bb8::Pool<RedisConnectionManager>,
    pub guild_ids: HashSet<u64>,
    pub shard: ShardState,
}

impl ShardManager {
    pub fn new(redis: bb8::Pool<RedisConnectionManager>, shard_id: u32) -> Self {
        Self {
            redis,
            guild_ids: HashSet::new(),
            shard: ShardState::new(shard_id),
        }
    }

    pub async fn handle_event(
        &mut self,
        event: Event,
        latency: &Latency,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match event {
            Event::GatewayHello(hello) => self.helloed(hello).await,
            Event::Ready(info) => self.readied(*info).await,
            Event::GuildCreate(created) => self.guild_created(*created).await,
            Event::GuildDelete(deleted) => self.guild_deleted(deleted).await,
            Event::Resumed => self.resumed().await,
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

    async fn helloed(&mut self, hello: Hello) -> Result<(), Box<dyn std::error::Error>> {
        // heartbeat_interval is a u64, but should be within bounds of u32,
        // do error if it isn't for some reason
        self.shard.heartbeat_interval = u32::try_from(hello.heartbeat_interval)?;
        self.shard.last_connection = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time went backwards")
            .as_secs();

        self.save_shard().await
    }

    async fn readied(&mut self, ready: Ready) -> Result<(), Box<dyn std::error::Error>> {
        tracing::info!(
            "shard {} ready ({} guilds)",
            self.shard.shard_id,
            ready.guilds.len()
        );

        self.guild_ids
            .extend(ready.guilds.into_iter().map(|g| g.id.get()));

        self.shard.up = true;
        self.shard.guild_count = self
            .guild_ids
            .len()
            .try_into()
            .expect("couldn't convert len() to u64");

        self.save_shard().await
    }

    async fn resumed(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        tracing::info!("shard {} resumed", self.shard.shard_id,);

        self.shard.up = true;
        self.shard.last_connection = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time went backwards")
            .as_secs();

        self.save_shard().await
    }

    async fn guild_created(
        &mut self,
        created: GuildCreate,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if !self.guild_ids.insert(created.id.get()) {
            // guild was already in set, do nothing
            return Ok(());
        }

        self.shard.guild_count = self
            .guild_ids
            .len()
            .try_into()
            .expect("couldn't convert len() to u64");

        self.save_shard().await
    }

    async fn guild_deleted(
        &mut self,
        deleted: GuildDelete,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if !self.guild_ids.remove(&deleted.id.get()) {
            // guild wasn't in set, do nothing
            return Ok(());
        }

        self.shard.guild_count = self
            .guild_ids
            .len()
            .try_into()
            .expect("couldn't convert len() to u64");

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
