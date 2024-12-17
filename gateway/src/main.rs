use std::error::Error;

use twilight_model::gateway::{
    event::GatewayEventDeserializer,
    payload::outgoing::UpdatePresence,
    presence::{Activity, ActivityEmoji, MinimalActivity, Status},
    OpCode,
};

use tulpje_shared::DiscordEvent;

#[cfg(feature = "cache")]
mod cache;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // set-up logging
    tracing_subscriber::fmt::init();

    let token = std::env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN not set");

    // needed for fetching recommended shard count
    let proxy_address = std::env::var("DISCORD_PROXY").expect("DISCORD_PROXY not set");
    let client = twilight_http::Client::builder()
        .proxy(proxy_address.clone(), true)
        .ratelimiter(None)
        .build();

    let config = twilight_gateway::Config::builder(token, twilight_gateway::Intents::all()).build();
    let mut shard = twilight_gateway::Shard::with_config(twilight_gateway::ShardId::ONE, config);

    let rabbitmq_address = std::env::var("RABBITMQ_ADDRESS").expect("RABBITMQ_ADDRESS not set");
    let rabbitmq_options = lapin::ConnectionProperties::default()
        .with_executor(tokio_executor_trait::Tokio::current())
        .with_reactor(tokio_reactor_trait::Tokio);
    let rabbitmq_conn = lapin::Connection::connect(&rabbitmq_address, rabbitmq_options)
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

    // Create the cache
    #[cfg(feature = "cache")]
    let cache = redlight::RedisCache::<cache::Config>::new(
        &std::env::var("REDIS_URL").expect("REDIS_URL not set"),
    )
    .await?;

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

                tracing::debug!(?opcode, "event received");

                // only publish non-gateway events, aka everything DISPATCH
                if opcode == OpCode::Dispatch {
                    // Blindly assuming the first one of these is the Ready event soo
                    if !init_done.swap(true, std::sync::atomic::Ordering::SeqCst) {
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

                        if let Err(err) = set_presence(&mut shard, state, Status::Online).await {
                            tracing::error!(?err, "error setting presence");
                        }
                    }

                    let event = DiscordEvent::new(text);

                    rabbitmq_chan
                        .basic_publish(
                            "",
                            "discord",
                            lapin::options::BasicPublishOptions::default(),
                            &serde_json::to_vec(&event)?,
                            lapin::BasicProperties::default(),
                        )
                        .await?;

                    tracing::debug!(uuid=?event.uuid, "event sent");
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
