mod amqp;
mod config;
mod context;
mod db;
mod metrics;
mod modules;

use std::{sync::Arc, time::Duration};

use bb8_redis::RedisConnectionManager;
use context::Services;
use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions},
    ConnectOptions,
};
use tracing::log::LevelFilter;

use tulpje_framework::registry::Registry;
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

    // set-up metrics
    tracing::info!("installing metrics collector and exporter...");
    metrics::install().expect("error setting up metrics");

    // needed for fetching recommended shard count
    let client = twilight_http::Client::builder()
        .proxy(config.discord_proxy, true)
        .ratelimiter(None)
        .build();

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

    // create AMQP connection
    let mut amqp = amqp::create(&config.rabbitmq_address).await;

    tracing::info!("running migrations...");
    sqlx::migrate!("../migrations")
        .run(&db)
        .await
        .expect("error running migrations");

    // Client interaction client
    let app = client.current_user_application().await?.model().await?;
    let context = context::Context {
        application_id: app.id,
        services: context::Services { redis, db },
        client: Arc::new(client),
    };

    // register interaction handlers
    tracing::info!("registering handlers");
    let mut registry = Registry::<Services>::new(context.clone());
    modules::stats::setup(&mut registry).await;
    modules::emoji::setup(&mut registry).await;
    modules::pk::setup(&mut registry).await;

    // start the task scheduler
    let sched_handle = registry.task.run().await;

    tracing::info!("registering global commands");
    context
        .interaction()
        .set_global_commands(&registry.get_global_commands())
        .await?;

    let main_handle = tokio::spawn(async move {
        loop {
            let Some(message) = amqp.recv().await else {
                break;
            };

            let (meta, event) = match parse_delivery(message) {
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

            tulpje_framework::handle(meta, context.clone(), &mut registry, event.clone()).await;
        }
    });

    futures_util::future::join_all([main_handle, sched_handle]).await;

    Ok(())
}

fn parse_delivery(
    message: Vec<u8>,
) -> Result<(DiscordEventMeta, twilight_model::gateway::event::Event), Box<dyn std::error::Error>> {
    let discord_event = serde_json::from_str::<DiscordEvent>(&String::from_utf8(message)?)?;

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
