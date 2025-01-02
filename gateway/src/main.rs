use std::{env, error::Error};

use bb8_redis::RedisConnectionManager;
use futures_util::StreamExt;
use twilight_gateway::EventTypeFlags;
use twilight_model::gateway::{
    event::{Event, GatewayEventDeserializer},
    payload::outgoing::{identify::IdentifyProperties, update_presence::UpdatePresencePayload},
    presence::{Activity, MinimalActivity, Status},
    OpCode,
};

use tulpje_shared::DiscordEvent;

mod amqp;
#[cfg(feature = "cache")]
mod cache;
mod config;
mod metrics;
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

    // set-up metrics
    tracing::info!("installing metrics collector and exporter...");
    metrics::install(config.shard_id).expect("error setting up metrics");

    let amqp = amqp::create(&config.rabbitmq_address).await;

    // create the cache
    #[cfg(feature = "cache")]
    let cache = redlight::RedisCache::<cache::Config>::new(&config.redis_url).await?;

    // create the redis connection
    let manager = RedisConnectionManager::new(config.redis_url).expect("error initialising redis");
    let redis = bb8::Pool::builder()
        .build(manager)
        .await
        .expect("error initialising redis pool");

    // create the shard
    tracing::info!("shard: {}, total: {}", config.shard_id, config.shard_count);
    let shard_config = twilight_gateway::ConfigBuilder::new(
        config.discord_token,
        twilight_gateway::Intents::all(),
    )
    .presence(create_presence())
    .identify_properties(IdentifyProperties {
        browser: "tulpje".into(),
        device: "tulpje".into(),
        os: std::env::consts::OS.into(),
    })
    .build();
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

                if let Ok(Some(event)) =
                    twilight_gateway::parse(text.clone(), EventTypeFlags::all())
                {
                    let event = twilight_model::gateway::event::Event::from(event);

                    // track event metrics
                    metrics::track_gateway_event(shard_id.number(), &event);

                    if let Err(err) = shard_state_manager
                        .handle_event(event.clone(), shard.latency())
                        .await
                    {
                        tracing::error!("error updating shard state: {}", err);
                    }
                }

                // only publish non-gateway events, aka everything DISPATCH
                if opcode == OpCode::Dispatch {
                    let event = DiscordEvent::new(shard_id.number(), text);
                    let serialized_event = match serde_json::to_vec(&event) {
                        Ok(val) => val,
                        Err(err) => {
                            tracing::error!("error serializing event: {}", err);
                            continue;
                        }
                    };

                    if let Err(err) = amqp.send(&serialized_event).await {
                        tracing::error!("error sending event to amqp: {}", err);
                        continue;
                    }

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

fn create_presence() -> UpdatePresencePayload {
    let state = format!(
        " Version: {} ({}{})",
        env!("CARGO_PKG_VERSION"),
        env!("VERGEN_GIT_SHA"),
        match env!("VERGEN_GIT_DIRTY") {
            "true" => "-dirty",
            _ => "",
        }
    );

    let mut activity: Activity = MinimalActivity {
        kind: twilight_model::gateway::presence::ActivityType::Custom,
        name: "~".into(),
        url: None,
    }
    .into();
    activity.state = Some(state);

    UpdatePresencePayload::new(vec![activity], false, None, Status::Online)
        .expect("couldn't create UpdatePresence struct")
}

fn parse_opcode(event: &str) -> Result<Option<OpCode>, Box<dyn Error>> {
    let Some(gateway_deserializer) = GatewayEventDeserializer::from_json(event) else {
        return Err("couldn't deserialise event".into());
    };

    Ok(OpCode::from(gateway_deserializer.op()))
}
