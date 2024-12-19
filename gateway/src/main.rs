use std::{env, error::Error};

use bb8_redis::{redis::AsyncCommands, RedisConnectionManager};
use twilight_model::gateway::{
    event::Event,
    event::GatewayEventDeserializer,
    payload::outgoing::UpdatePresence,
    presence::{Activity, MinimalActivity, Status},
    OpCode,
};

use tulpje_shared::DiscordEvent;

#[cfg(feature = "cache")]
mod cache;
mod config;

use config::Config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // load .env into environment vars, ignore if not found
    match dotenvy::dotenv().map(|_| ()) {
        Err(err) if err.not_found() => {
            tracing::warn!("no .env file found");
            ()
        }
        result => result?,
    };

    // parse TASK_SLOT env var if it exists and use it for the shard id
    if let Ok(task_slot) = env::var("TASK_SLOT") {
        tracing::info!("TASK_SLOT env var found, using it for shard id");
        tracing::debug!("TASK_SLOT = {}", task_slot);

        env::set_var(
            "shard_id",
            format!(
                "{}",
                task_slot.parse::<u64>().expect("couldn't parse task_slot") - 1
            ),
        );
    }

    // create config from environment vars
    let config = Config::from_env()?;

    // set-up logging
    tracing_subscriber::fmt::init();

    // create the rabbitmq connection
    let rabbitmq_options = lapin::ConnectionProperties::default()
        .with_executor(tokio_executor_trait::Tokio::current())
        .with_reactor(tokio_reactor_trait::Tokio);
    let rabbitmq_conn = lapin::Connection::connect(&config.rabbitmq_address, rabbitmq_options)
        .await
        .expect("couldn't connec to RabbitMQ");
    let rabbitmq_chan = rabbitmq_conn
        .create_channel()
        .await
        .expect("couldn't create RabbitMQ channel");

    // declare the queue
    rabbitmq_chan
        .queue_declare(
            "discord",
            lapin::options::QueueDeclareOptions {
                durable: true,
                ..Default::default()
            },
            lapin::types::FieldTable::default(),
        )
        .await
        .expect("couldn't declare queue");

    // create the redis connection
    let manager = RedisConnectionManager::new(config.redis_url).expect("error initialising redis");
    let redis = bb8::Pool::builder()
        .build(manager)
        .await
        .expect("error initialising redis pool");

    // create the cache
    #[cfg(feature = "cache")]
    let cache = redlight::RedisCache::<cache::Config>::new(&config.redis_url).await?;

    // create discord api client
    let client = twilight_http::Client::builder()
        .proxy(config.discord_proxy, true)
        .ratelimiter(None)
        .build();

    // create the shard
    tracing::info!("shard: {}, total: {}", config.shard_id, config.shard_count);
    let shard_config =
        twilight_gateway::Config::builder(config.discord_token, twilight_gateway::Intents::all())
            .build();
    let shard_id = twilight_gateway::ShardId::new_checked(config.shard_id, config.shard_count)
        .expect("error constructing shard ID");
    let mut shard = twilight_gateway::Shard::with_config(shard_id, shard_config);

    // initialisation done, ratelimit on session_limit
    tracing::info!("waiting for gateway queue...");
    reqwest::get(config.discord_gateway_queue).await?;

    // start main loop
    tracing::info!("starting main loop...");
    let init_done = std::sync::atomic::AtomicBool::new(false);
    loop {
        match shard.next_message().await {
            Ok(twilight_gateway::Message::Close(frame)) => {
                tracing::warn!(?frame, "Message::Close");
            }
            Ok(twilight_gateway::Message::Text(text)) => {
                let opcode = match parse_opcode(&text) {
                    Err(err) => {
                        tracing::error!(?err, "couldn't parse opcode");
                        continue;
                    }
                    Ok(Some(opcode)) => opcode,
                    Ok(None) => {
                        tracing::error!("received empty opcode");
                        continue;
                    }
                };

                tracing::debug!(?opcode, "opcode received");

                if let Ok(Some(event)) = twilight_gateway::parse(
                    text.clone(),
                    twilight_gateway::EventTypeFlags::GATEWAY_HEARTBEAT_ACK
                        | twilight_gateway::EventTypeFlags::READY,
                ) {
                    match event.into() {
                        Event::GatewayHeartbeatAck => handle_ack(&redis, &shard).await,
                        Event::Ready(_bot_info) => {
                            // we only run global init code on the first shard
                            if shard_id.number() == 0 {
                                if !init_done.swap(true, std::sync::atomic::Ordering::SeqCst) {
                                    handle_first_ready(&mut shard).await
                                } else {
                                    tracing::warn!(
                                        "Event::Ready fired a second time on first shard"
                                    );
                                }
                            }
                        }
                        _ => {}
                    }
                }

                // only publish non-gateway events, aka everything DISPATCH
                if opcode == OpCode::Dispatch {
                    let event = DiscordEvent::new(shard_id.number(), text);

                    rabbitmq_chan
                        .basic_publish(
                            "",
                            "discord",
                            lapin::options::BasicPublishOptions::default(),
                            &serde_json::to_vec(&event)?,
                            lapin::BasicProperties::default(),
                        )
                        .await?;

                    tracing::debug!(
                        uuid = ?event.meta.uuid,
                        shard = event.meta.shard,
                        "event sent"
                    );
                }
            }
            Err(err) => {
                tracing::error!(?err, "error receiving discord message");
            }
        };
    }
}

async fn set_presence(
    shard: &mut twilight_gateway::Shard,
    state: String,
    status: Status,
) -> Result<(), Box<dyn Error>> {
    let mut activity: Activity = MinimalActivity {
        kind: twilight_model::gateway::presence::ActivityType::Custom,
        name: "~".into(),
        url: None,
    }
    .into();
    activity.state = Some(state.into());

    shard
        .send(serde_json::to_string(&UpdatePresence::new(
            vec![activity],
            false,
            None,
            status,
        )?)?)
        .await?;

    Ok(())
}

fn parse_opcode(event: &String) -> Result<Option<OpCode>, Box<dyn Error>> {
    let Some(gateway_deserializer) = GatewayEventDeserializer::from_json(event) else {
        return Err("couldn't deserialise event".into());
    };

    Ok(OpCode::from(gateway_deserializer.op()))
}

async fn handle_ack(redis: &bb8::Pool<RedisConnectionManager>, shard: &twilight_gateway::Shard) {
    let mut conn = redis.get().await.expect("error acquiring redis connection");

    let shard_id = shard.id().number();
    let current_latency = match shard.latency().recent().first() {
        Some(latency) => latency,
        None => {
            tracing::error!("latency().recent().first() is empty, shouldn't happen after an ack");
            return;
        }
    };
    let average_latency = match shard.latency().average() {
        Some(latency) => latency,
        None => {
            tracing::error!("latency().average() is empty, shouldn't happen after an ack");
            return;
        }
    };
    tracing::info!(
        ?current_latency,
        ?average_latency,
        shard_id,
        "HeartbeatAck received"
    );

    if let Err(err) = conn
        .hset_multiple::<&str, std::string::String, u64, ()>(
            "shards",
            &[
                (
                    format!("latency_curr_{}", shard_id),
                    current_latency.as_millis() as u64,
                ),
                (
                    format!("latency_avg_{}", shard_id),
                    average_latency.as_millis() as u64,
                ),
            ],
        )
        .await
    {
        tracing::error!("error storing latency stats: {}", err);
    } else {
        tracing::trace!(shard_id, "latency info stored in db")
    }
}

async fn handle_first_ready(shard: &mut twilight_gateway::Shard) {
    tracing::debug!("setting presence");

    let state = format!(
        " Version: {} ({}{})",
        env!("CARGO_PKG_VERSION"),
        env!("VERGEN_GIT_SHA"),
        match env!("VERGEN_GIT_DIRTY") {
            "true" => "-dirty",
            _ => "",
        }
    );

    if let Err(err) = set_presence(shard, state, Status::Online).await {
        tracing::error!(?err, "error setting presence");
    }
}
