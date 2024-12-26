pub mod commands;
pub mod db;
pub mod event_handler;
pub mod shared;

use twilight_gateway::EventType;
use twilight_model::{application::command::CommandType, guild::Permissions};
use twilight_util::builder::command::{CommandBuilder, StringBuilder};

use tulpje_framework::{
    handler::{
        command_handler::CommandHandler,
        component_interaction_handler::ComponentInteractionHandler, event_handler::EventHandler,
    },
    registry::Registry,
};

use crate::context::Services;

pub fn setup(registry: &mut Registry<Services>) {
    // commands
    registry.command.insert(CommandHandler {
        definition: CommandBuilder::new(
            "emoji-stats",
            "Stats for emojis in this server",
            CommandType::ChatInput,
        )
        .default_member_permissions(Permissions::MANAGE_GUILD_EXPRESSIONS)
        .dm_permission(false)
        .option(
            StringBuilder::new("sort", "How to sort the emojis")
                .choices([
                    ("Most Used", "count_desc"),
                    ("Least Used", "count_asc"),
                    ("Most Recent", "date_desc"),
                    ("Least Recent", "date_asc"),
                ])
                .build(),
        )
        .build(),
        func: |ctx| Box::pin(commands::cmd_emoji_stats(ctx)),
    });

    // component interactions
    registry
        .component_interaction
        .insert(ComponentInteractionHandler {
            custom_id: "emoji_stats_sort".into(),
            func: |ctx| Box::pin(commands::handle_emoji_stats_sort(ctx)),
        });

    // event handlers
    registry.event.insert(EventHandler {
        uuid: uuid::Uuid::now_v7().to_string(),
        event: EventType::MessageCreate,
        func: |ctx| Box::pin(event_handler::handle_message(ctx)),
    });
    registry.event.insert(EventHandler {
        uuid: uuid::Uuid::now_v7().to_string(),
        event: EventType::MessageUpdate,
        func: |ctx| Box::pin(event_handler::message_update(ctx)),
    });
    registry.event.insert(EventHandler {
        uuid: uuid::Uuid::now_v7().to_string(),
        event: EventType::ReactionAdd,
        func: |ctx| Box::pin(event_handler::reaction_add(ctx)),
    });
}
