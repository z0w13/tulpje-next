use std::{collections::HashMap, sync::Arc};

use twilight_gateway::{Event, EventTypeFlags};
use twilight_http::client::ClientBuilder;
use twilight_model::{
    application::command::CommandType,
    id::{marker::ApplicationMarker, Id},
};
use twilight_util::builder::command::CommandBuilder;

use tulpje_framework::{
    context::{CommandContext, TaskContext},
    handler::task_handler::TaskHandler,
    handler_func, Context, Error, ModuleBuilder, Registry,
};
use tulpje_shared::DiscordEvent;

#[derive(Clone)]
#[expect(dead_code, reason = "testing")]
struct UserData {
    tasks: Arc<HashMap<String, TaskHandler<UserData>>>,
}

async fn command_func(_ctx: CommandContext<UserData>) -> Result<(), Error> {
    Ok(())
}
async fn task_func(_ctx: TaskContext<UserData>) -> Result<(), Error> {
    Ok(())
}

const EVENT_JSON: &'static str = r###"
    {
      "t": "INTERACTION_CREATE",
      "s": 3,
      "op": 0,
      "d": {
        "version": 1,
        "type": 2,
        "token": "empty",
        "member": {
          "user": {
            "username": "username",
            "public_flags": 0,
            "primary_guild": null,
            "id": "1",
            "global_name": "global_name",
            "discriminator": "0",
            "clan": null,
            "avatar_decoration_data": null,
            "avatar": "00000000000000000000000000000000"
          },
          "unusual_dm_activity_until": null,
          "roles": [ "1" ],
          "premium_since": null,
          "permissions": "1",
          "pending": false,
          "nick": "nick",
          "mute": false,
          "joined_at": "2020-03-10T00:00:00.000000+00:00",
          "flags": 0,
          "deaf": false,
          "communication_disabled_until": null,
          "banner": null,
          "avatar": null
        },
        "locale": "en-US",
        "id": "1",
        "guild_locale": "en-US",
        "guild_id": "1",
        "guild": {
          "locale": "en-US",
          "id": "1",
          "features": [
            "ENABLED_MODERATION_EXPERIENCE_FOR_NON_COMMUNITY",
            "COMMUNITY",
            "NEWS"
          ]
        },
        "entitlements": [],
        "entitlement_sku_ids": [],
        "data": {
          "type": 1,
          "options": [ ],
          "name": "__COMMAND__",
          "id": "1"
        },
        "context": 0,
        "channel_id": "1",
        "channel": {
          "type": 0,
          "topic": null,
          "theme_color": null,
          "rate_limit_per_user": 0,
          "position": 2,
          "permissions": "1",
          "parent_id": "1",
          "nsfw": false,
          "name": "general",
          "last_message_id": "1",
          "id": "1",
          "icon_emoji": {
            "name": "👋",
            "id": null
          },
          "guild_id": "1",
          "flags": 0
        },
        "authorizing_integration_owners": {
          "0": "1"
        },
        "application_id": "1",
        "app_permissions": "1"
      }
    }
"###;

#[tokio::main]
async fn main() {
    let client = ClientBuilder::new().build();

    let mut registry = Registry::<UserData>::new();
    let mut builder = ModuleBuilder::<UserData>::new("bench");

    for num in 0..10_000 {
        builder = builder.command(
            CommandBuilder::new(
                format!("command-name-{}", num),
                "desc",
                CommandType::ChatInput,
            )
            .build(),
            handler_func!(command_func),
        );
        builder = builder.task(
            &format!("task-name-{}", num),
            "* * * * * *",
            handler_func!(task_func),
        );
    }
    registry.register(builder.build());

    let ctx = Context {
        services: UserData {
            tasks: Arc::new(registry.tasks.clone()),
        },
        application_id: Id::<ApplicationMarker>::new(1),
        client: Arc::new(client),
    };

    let event = DiscordEvent::new(
        1,
        EVENT_JSON
            .replace("__COMMAND__", "command-name-500")
            .to_owned(),
    );
    let discord_event = Event::from(
        twilight_gateway::parse(event.payload, EventTypeFlags::all())
            .expect("Couldn't parse payload")
            .expect("Payload is None"),
    );

    let total_iterations = 1_000;
    for iteration in 0..total_iterations {
        if iteration % 100 == 0 {
            println!("iteration {} of {}", iteration, total_iterations);
        }

        tulpje_framework::handle(
            event.meta.clone(),
            ctx.clone(),
            &registry,
            discord_event.clone(),
        )
        .await;
    }
}
