mod commands;
mod config;
mod context;

use std::sync::Arc;

use bb8_redis::RedisConnectionManager;
use futures::StreamExt;

use tulpje_shared::{DiscordEvent, DiscordEventMeta};

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

    // create config from environment vars
    let config = Config::from_env()?;

    // set-up logging
    tracing_subscriber::fmt::init();

    // needed for fetching recommended shard count
    let client = twilight_http::Client::builder()
        .proxy(config.discord_proxy, true)
        .ratelimiter(None)
        .build();

    let rabbitmq_options = lapin::ConnectionProperties::default()
        .with_executor(tokio_executor_trait::Tokio::current())
        .with_reactor(tokio_reactor_trait::Tokio);
    let rabbitmq_conn =
        lapin::Connection::connect(&config.rabbitmq_address, rabbitmq_options).await?;
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

    let mut rabbitmq_consumer = rabbitmq_chan
        .basic_consume(
            "discord",
            "handler",
            lapin::options::BasicConsumeOptions {
                no_ack: true,
                ..Default::default()
            },
            lapin::types::FieldTable::default(),
        )
        .await?;

    // Client interaction client
    let app = client.current_user_application().await?.model().await?;
    let context = Arc::new(context::Context {
        application_id: app.id,
        services: context::Services { redis },
        client,
    });
    let interaction = context.client.interaction(app.id);

    // register commands
    tracing::info!("registering commands");
    interaction.set_global_commands(&commands::build()).await?;

    loop {
        let Some(delivery) = rabbitmq_consumer.next().await else {
            break;
        };

        let (meta, event) = match parse_delivery(delivery) {
            Ok((meta, event)) => (meta, event),
            Err(err) => {
                tracing::error!(?err, "couldn't parse delivery");
                continue;
            }
        };

        tracing::debug!(
            event = ?event.kind(),
            uuid = ?meta.uuid,
            shard = meta.shard,
            "event received",
        );

        match event {
            twilight_gateway::Event::InteractionCreate(event) => {
                tracing::info!("interaction");

                // handle a command
                let Some(
                    twilight_model::application::interaction::InteractionData::ApplicationCommand(
                        command,
                    ),
                ) = &event.data
                else {
                    tracing::info!("Not an application command");
                    continue;
                };

                let command_context = context::CommandContext {
                    meta,
                    context: context.clone(),
                    command: *command.clone(),
                    event: *event.clone(),
                };

                tracing::info!("processing command /{}", command.name);
                if let Err(err) = commands::handle(command_context).await {
                    tracing::warn!("error processing command /{}: {}", command.name, err);
                };
            }
            e => tracing::warn!(event = ?e.kind(), "unhandled event"),
        }
    }

    Ok(())
}

fn parse_delivery(
    delivery: Result<lapin::message::Delivery, lapin::Error>,
) -> Result<(DiscordEventMeta, twilight_model::gateway::event::Event), Box<dyn std::error::Error>> {
    let message = delivery?;

    let discord_event = serde_json::from_str::<DiscordEvent>(&String::from_utf8(message.data)?)?;

    Ok((
        discord_event.meta,
        twilight_gateway::Event::from(
            twilight_gateway::parse(
                discord_event.payload,
                twilight_gateway::EventTypeFlags::all(),
            )?
            .unwrap(),
        ),
    ))
}
