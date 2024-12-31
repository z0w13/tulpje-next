use std::{env, error::Error};

use bb8_redis::RedisConnectionManager;
use futures_util::StreamExt;
use twilight_gateway::EventTypeFlags;
use twilight_model::gateway::{
    event::Event,
    event::GatewayEventDeserializer,
    payload::outgoing::UpdatePresence,
    presence::{Activity, MinimalActivity, Status},
    OpCode,
};

use tulpje_shared::DiscordEvent;

mod amqp;
#[cfg(feature = "cache")]
mod cache;
mod config;
mod shard_state;

use config::Config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // load .env into environment vars, ignore if not found
    match dotenvy::dotenv().map(|_| ()) {
        Err(err) if err.not_found() => {
            tracing::warn!("no .env file found");
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

    let amqp = amqp::create(&config.rabbitmq_address).await;

    // create the redis connection
    let manager = RedisConnectionManager::new(config.redis_url).expect("error initialising redis");
    let redis = bb8::Pool::builder()
        .build(manager)
        .await
        .expect("error initialising redis pool");

    // create the cache
    #[cfg(feature = "cache")]
    let cache = redlight::RedisCache::<cache::Config>::new(&config.redis_url).await?;

    // create the shard
    tracing::info!("shard: {}, total: {}", config.shard_id, config.shard_count);
    let shard_config =
        twilight_gateway::Config::new(config.discord_token, twilight_gateway::Intents::all());
    let shard_id = twilight_gateway::ShardId::new_checked(config.shard_id, config.shard_count)
        .expect("error constructing shard ID");
    let mut shard = twilight_gateway::Shard::with_config(shard_id, shard_config);

    // create shard state manager
    let mut shard_state_manager = shard_state::ShardManager::new(redis.clone(), shard_id.number());

    // initialisation done, ratelimit on session_limit
    tracing::info!("waiting for gateway queue...");
    reqwest::get(config.discord_gateway_queue).await?;

    // start main loop
    tracing::info!("starting main loop...");
    let init_done = std::sync::atomic::AtomicBool::new(false);
    loop {
        match shard.next().await {
            Some(Ok(twilight_gateway::Message::Close(frame))) => {
                tracing::warn!(?frame, "gateway connection closed");

                // have to handle this hear separate as twilight_gateway::parse doesn't
                // parse into Event::GatewayClose as that's a separate event type
                if let Err(err) = shard_state_manager
                    .handle_event(Event::GatewayClose(frame), shard.latency())
                    .await
                {
                    tracing::error!("error updating shard state: {}", err);
                }
            }
            Some(Ok(twilight_gateway::Message::Text(text))) => {
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
                    EventTypeFlags::GATEWAY_HEARTBEAT_ACK
                        | EventTypeFlags::GATEWAY_HELLO
                        | EventTypeFlags::GUILD_CREATE
                        | EventTypeFlags::GUILD_DELETE
                        | EventTypeFlags::RESUMED
                        | EventTypeFlags::READY,
                ) {
                    if let Err(err) = shard_state_manager
                        .handle_event(event.clone().into(), shard.latency())
                        .await
                    {
                        tracing::error!("error updating shard state: {}", err);
                    }

                    #[expect(clippy::single_match, reason = "this might be expanded in the future")]
                    match event.into() {
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

                    amqp.send(&serde_json::to_vec(&event)?).await?;

                    tracing::debug!(
                        uuid = ?event.meta.uuid,
                        shard = event.meta.shard,
                        "event sent"
                    );
                }
            }
            Some(Err(err)) => {
                tracing::error!(?err, "error receiving discord message");
            }
            None => {
                tracing::error!("received empty message");
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
    activity.state = Some(state);

    shard.send(serde_json::to_string(&UpdatePresence::new(
        vec![activity],
        false,
        None,
        status,
    )?)?);

    Ok(())
}

fn parse_opcode(event: &str) -> Result<Option<OpCode>, Box<dyn Error>> {
    let Some(gateway_deserializer) = GatewayEventDeserializer::from_json(event) else {
        return Err("couldn't deserialise event".into());
    };

    Ok(OpCode::from(gateway_deserializer.op()))
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
