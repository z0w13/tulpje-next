use tulpje_framework::handler::command_handler::CommandHandler;
use tulpje_framework::registry::Registry;
use tulpje_framework::{guild_command, task};
use tulpje_framework::handler::task_handler::TaskHandler;
use twilight_model::{application::command::CommandType, guild::Permissions};
use twilight_util::builder::command::{CommandBuilder, StringBuilder};

use crate::context::Services;

pub mod commands;
pub mod db;
pub mod fronters;
pub mod roles;
pub mod util;

pub async fn setup(registry: &mut Registry<Services>) {
    guild_command!(
        registry,
        "pluralkit",
        CommandBuilder::new(
            "setup-pk",
            "set-up the PluralKit module",
            CommandType::ChatInput
        )
        .default_member_permissions(Permissions::MANAGE_GUILD)
        .dm_permission(false)
        .option(
            StringBuilder::new("system_id", "PluralKit system ID")
                .required(true)
                .build()
        )
        .option(StringBuilder::new("token", "(optional) PluralKit token").build())
        .build(),
        commands::setup_pk
    );
    guild_command!(
        registry,
        "pluralkit",
        CommandBuilder::new(
            "setup-fronters",
            "set-up fronter channels",
            CommandType::ChatInput
        )
        .default_member_permissions(Permissions::MANAGE_GUILD)
        .dm_permission(false)
        .option(StringBuilder::new("name", "Name of the fronters category").build())
        .build(),
        fronters::commands::setup_fronters
    );
    guild_command!(
        registry,
        "pluralkit",
        CommandBuilder::new(
            "update-fronters",
            "manually update fronter channels",
            CommandType::ChatInput
        )
        .default_member_permissions(Permissions::MANAGE_GUILD)
        .dm_permission(false)
        .build(),
        fronters::commands::update_fronters
    );
    guild_command!(
        registry,
        "pluralkit",
        CommandBuilder::new(
            "update-member-roles",
            "update the member roles",
            CommandType::ChatInput
        )
        .default_member_permissions(Permissions::MANAGE_GUILD)
        .dm_permission(false)
        .build(),
        roles::update_member_roles
    );

    task!(
        registry,
        "pk:update-fronters",
        "0 * * * * *", // every minute
        fronters::tasks::update_fronters,
    );
}
