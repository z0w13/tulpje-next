mod config;
mod context;
mod modules;

use std::{sync::Arc, time::Duration};

use bb8_redis::RedisConnectionManager;
use futures::StreamExt;
use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions},
    ConnectOptions,
};
use tracing::log::LevelFilter;
use twilight_model::application::command::Command;
use twilight_model::application::interaction::InteractionData;

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

    // create the redis connection
    let manager = RedisConnectionManager::new(config.redis_url).expect("error initialising redis");
    let redis = bb8::Pool::builder()
        .build(manager)
        .await
        .expect("error initialising redis pool");

    // create postgres connection
    let connect_opts = config
        .database_url
        .parse::<PgConnectOptions>()
        .unwrap_or_else(|_| panic!("couldn't parse db url: {}", config.database_url))
        .log_statements(LevelFilter::Trace)
        .log_slow_statements(LevelFilter::Warn, Duration::from_secs(5));
    let db = PgPoolOptions::new()
        .max_connections(5)
        .connect_with(connect_opts)
        .await
        .expect("error connecting to db");

    tracing::info!("running migrations...");
    sqlx::migrate!("../migrations")
        .run(&db)
        .await
        .expect("error running migrations");

    // Client interaction client
    let app = client.current_user_application().await?.model().await?;
    let context = Arc::new(context::Context {
        application_id: app.id,
        services: context::Services { redis, db },
        client,
    });
    let interaction = context.client.interaction(app.id);

    // register commands
    tracing::info!("registering commands");
    interaction
        .set_global_commands(
            &vec![modules::stats::commands(), modules::emoji::commands()]
                .into_iter()
                .flatten()
                .collect::<Vec<Command>>(),
        )
        .await?;

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

        match event.clone() {
            twilight_gateway::Event::InteractionCreate(event) => {
                tracing::info!("interaction");

                match &event.data {
                    Some(InteractionData::ApplicationCommand(command)) => {
                        let command_context = context::CommandContext {
                            meta,
                            context: context.clone(),
                            command: *command.clone(),
                            event: *event.clone(),
                        };

                        tracing::info!("processing commmand /{}", command.name);

                        if let Err(err) =
                            modules::stats::handle_command(command_context.clone()).await
                        {
                            tracing::warn!("error processing command /{}: {}", command.name, err);
                        };

                        if let Err(err) =
                            modules::emoji::handle_command(command_context.clone()).await
                        {
                            tracing::warn!("error processing command /{}: {}", command.name, err);
                        };
                    }
                    Some(InteractionData::MessageComponent(interaction)) => {
                        let component_interaction_context = context::ComponentInteractionContext {
                            meta,
                            context: context.clone(),
                            interaction: *interaction.clone(),
                            event: *event.clone(),
                        };

                        if let Err(err) = modules::emoji::commands::handle_emoji_stats_sort(
                            component_interaction_context.clone(),
                        )
                        .await
                        {
                            tracing::warn!(
                                "error processing interaction {}: {}",
                                interaction.custom_id,
                                err
                            );
                        };
                    }
                    _ => (),
                }
            }
            e => tracing::warn!(event = ?e.kind(), "unhandled event"),
        }

        if let Err(err) =
            modules::emoji::event_handler::handle_event(context.clone(), event.clone()).await
        {
            tracing::warn!("modules::emoji::event_handler::handle_event: {}", err);
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
