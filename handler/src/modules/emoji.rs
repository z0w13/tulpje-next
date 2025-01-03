pub mod clone;
pub mod commands;
pub mod db;
pub mod event_handlers;
pub mod shared;

use twilight_gateway::EventType;
use twilight_model::{application::command::CommandType, guild::Permissions};
use twilight_util::builder::command::{CommandBuilder, StringBuilder};

use tulpje_framework::{
    command, component_interaction, event_handler,
    handler::{
        command_handler::CommandHandler,
        component_interaction_handler::ComponentInteractionHandler, event_handler::EventHandler,
    },
    registry::Registry,
};

use crate::context::Services;

pub async fn setup(registry: &mut Registry<Services>) {
    // commands
    command!(
        registry,
        CommandBuilder::new(
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
        commands::cmd_emoji_stats,
    );
    command!(
        registry,
        CommandBuilder::new(
            "emoji-clone",
            "clone an emoji to this server",
            CommandType::ChatInput
        )
        .default_member_permissions(Permissions::MANAGE_GUILD_EXPRESSIONS)
        .dm_permission(false)
        .option(
            StringBuilder::new("emoji", "emojis to clone")
                .required(true)
                .build()
        )
        .option(StringBuilder::new("new_name", "new name (only if cloning a single emoji)").build())
        .option(StringBuilder::new("prefix", "prefix for new emoji(s)").build())
        .build(),
        clone::command,
    );
    command!(
        registry,
        CommandBuilder::new("Clone Emojis", "", CommandType::Message)
            .default_member_permissions(Permissions::MANAGE_GUILD_EXPRESSIONS)
            .dm_permission(false)
            .build(),
        clone::context_command
    );

    // component interactions
    component_interaction!(
        registry,
        "emoji_stats_sort",
        commands::handle_emoji_stats_sort
    );

    // event handlers
    event_handler!(
        registry,
        EventType::MessageCreate,
        event_handlers::handle_message,
    );
    event_handler!(
        registry,
        EventType::MessageUpdate,
        event_handlers::message_update,
    );
    event_handler!(
        registry,
        EventType::ReactionAdd,
        event_handlers::reaction_add,
    );
}
