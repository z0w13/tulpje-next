pub mod clone;
pub mod commands;
pub mod db;
pub mod event_handlers;
pub mod shared;

use twilight_gateway::EventType;
use twilight_model::{application::command::CommandType, guild::Permissions};
use twilight_util::builder::command::{CommandBuilder, StringBuilder};

use tulpje_framework::{handler_func, Module, ModuleBuilder};

use crate::context::Services;

pub(crate) fn build() -> Module<Services> {
    ModuleBuilder::<Services>::new("emoji")
        // commands
        .command(
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
            handler_func!(commands::cmd_emoji_stats),
        )
        .command(
            CommandBuilder::new(
                "emoji-clone",
                "clone an emoji to this server",
                CommandType::ChatInput,
            )
            .default_member_permissions(Permissions::MANAGE_GUILD_EXPRESSIONS)
            .dm_permission(false)
            .option(
                StringBuilder::new("emoji", "emojis to clone")
                    .required(true)
                    .build(),
            )
            .option(
                StringBuilder::new("new_name", "new name (only if cloning a single emoji)").build(),
            )
            .option(StringBuilder::new("prefix", "prefix for new emoji(s)").build())
            .build(),
            handler_func!(clone::command),
        )
        .command(
            CommandBuilder::new("Clone Emojis", "", CommandType::Message)
                .default_member_permissions(Permissions::MANAGE_GUILD_EXPRESSIONS)
                .dm_permission(false)
                .build(),
            handler_func!(clone::context_command),
        )
        // component interactions
        .component(
            "emoji_stats_sort",
            handler_func!(commands::handle_emoji_stats_sort),
        )
        // event handlers
        .event(
            EventType::MessageCreate,
            handler_func!(event_handlers::handle_message),
        )
        .event(
            EventType::MessageUpdate,
            handler_func!(event_handlers::message_update),
        )
        .event(
            EventType::ReactionAdd,
            handler_func!(event_handlers::reaction_add),
        )
        .build()
}
