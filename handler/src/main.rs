mod config;

use futures::StreamExt;

use tulpje_shared::{DiscordEvent, DiscordEventMeta};

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

    // Client interaction client
    let app = client.current_user_application().await?.model().await?;
    let interaction = client.interaction(app.id);

    // register a command
    // tracing::info!("registering /twilight-test");
    // let command = twilight_util::builder::command::CommandBuilder::new(
    //     "twilight-test",
    //     "test command",
    //     twilight_model::application::command::CommandType::ChatInput,
    // )
    // .dm_permission(false)
    // .build();
    // interaction.set_global_commands(&[]).await?;
    // interaction.set_global_commands(&[command]).await?;

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
            twilight_gateway::Event::InteractionCreate(e) => {
                tracing::info!(interaction = ?e, "interaction");

                // handle a command
                let Some(
                    twilight_model::application::interaction::InteractionData::ApplicationCommand(
                        command,
                    ),
                ) = &e.data
                else {
                    tracing::info!("Not an application command");
                    continue;
                };

                tracing::info!(command = command.name, "command");

                let response = twilight_util::builder::InteractionResponseDataBuilder::new()
                    .content("ohey")
                    .flags(twilight_model::channel::message::MessageFlags::EPHEMERAL)
                    .build();

                if let Err(err) = interaction
                    .create_response(
                        e.id,
                        &e.token,
                        &twilight_model::http::interaction::InteractionResponse {
                            kind: twilight_model::http::interaction::InteractionResponseType::ChannelMessageWithSource,
                            data: Some(response),
                        },
                    )
                    .await
                {
                    tracing::warn!(?err, "failed to respond to command")
                }
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
