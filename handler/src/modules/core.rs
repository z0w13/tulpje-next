use std::collections::HashMap;

use twilight_http::client::InteractionClient;
use twilight_model::{
    application::command::{Command, CommandType},
    guild::Permissions,
    id::{marker::GuildMarker, Id},
};
use twilight_util::builder::command::{CommandBuilder, StringBuilder};

use tulpje_framework::{handler_func, Error, Module, ModuleBuilder, Registry};

use crate::{
    context::{CommandContext, Services},
    db::DbId,
};

pub(crate) fn build(registry: &Registry<Services>) -> Module<Services> {
    let guild_module_choices: Vec<(String, String)> = registry
        .guild_module_names()
        .into_iter()
        .map(|m| (m.clone(), m))
        .collect();

    ModuleBuilder::<Services>::new("core")
        .command(
            CommandBuilder::new(
                "enable",
                "enable a module for this server",
                CommandType::ChatInput,
            )
            .default_member_permissions(Permissions::MANAGE_GUILD)
            .dm_permission(false)
            .option(
                StringBuilder::new("module", "The module to enable")
                    .choices(guild_module_choices.clone())
                    .required(true)
                    .build(),
            )
            .build(),
            handler_func!(enable),
        )
        .command(
            CommandBuilder::new(
                "disable",
                "disable a module for this server",
                CommandType::ChatInput,
            )
            .default_member_permissions(Permissions::MANAGE_GUILD)
            .dm_permission(false)
            .option(
                StringBuilder::new("module", "The module to disable")
                    .choices(guild_module_choices)
                    .required(true)
                    .build(),
            )
            .build(),
            handler_func!(disable),
        )
        .command(
            CommandBuilder::new(
                "modules",
                "list enabled and available server modules",
                CommandType::ChatInput,
            )
            .default_member_permissions(Permissions::MANAGE_GUILD)
            .dm_permission(false)
            .build(),
            handler_func!(modules),
        )
        .build()
}

pub(crate) async fn enable(ctx: CommandContext) -> Result<(), Error> {
    let Some(guild) = ctx.guild().await? else {
        unreachable!("command is guild_only");
    };

    let module = ctx.get_arg_string("module")?;
    if !ctx.services.registry.guild_module_names().contains(&module) {
        ctx.reply(format!("invalid module {}", module)).await?;
        return Ok(());
    }

    db_enable_module(&ctx.services.db, guild.id, &module).await?;
    set_guild_commands_for_guild(
        db_guild_modules(&ctx.services.db, guild.id).await?,
        guild.id,
        ctx.interaction(),
        &ctx.services.registry,
    )
    .await?;

    ctx.reply(format!("{} enabled", module)).await?;

    Ok(())
}

pub(crate) async fn disable(ctx: CommandContext) -> Result<(), Error> {
    let Some(guild) = ctx.guild().await? else {
        unreachable!("command is guild_only");
    };

    let module = ctx.get_arg_string("module")?;
    if !ctx.services.registry.guild_module_names().contains(&module) {
        ctx.reply(format!("invalid module {}", module)).await?;
        return Ok(());
    }

    db_disable_module(&ctx.services.db, guild.id, &module).await?;
    set_guild_commands_for_guild(
        db_guild_modules(&ctx.services.db, guild.id).await?,
        guild.id,
        ctx.interaction(),
        &ctx.services.registry,
    )
    .await?;

    ctx.reply(format!("{} disabled", module)).await?;

    Ok(())
}

pub(crate) async fn modules(ctx: CommandContext) -> Result<(), Error> {
    let Some(guild) = ctx.guild().await? else {
        unreachable!("command is guild_only");
    };

    let modules = db_guild_modules(&ctx.services.db, guild.id).await?;
    let available: Vec<String> = ctx
        .services
        .registry
        .guild_module_names()
        .into_iter()
        .filter(|m| !modules.contains(m))
        .collect();

    ctx.reply(format!(
        "**Enabled: {}**\nAvailable: {}",
        modules.join(", "),
        available.join(", ")
    ))
    .await?;

    Ok(())
}

pub(crate) async fn set_guild_commands_for_guild(
    modules: Vec<String>,
    guild_id: Id<GuildMarker>,
    interaction: InteractionClient<'_>,
    registry: &Registry<Services>,
) -> Result<(), Error> {
    let commands: Vec<Command> = modules
        .iter()
        .filter_map(|module| registry.module_commands(module))
        .flatten()
        .collect();

    tracing::debug!(
        "setting commands [{}] for guild {}",
        commands
            .iter()
            .map(|cmd| cmd.name.clone())
            .collect::<Vec<String>>()
            .join(", "),
        guild_id
    );

    interaction.set_guild_commands(guild_id, &commands).await?;

    Ok(())
}

async fn db_enable_module(
    db: &sqlx::PgPool,
    guild_id: Id<GuildMarker>,
    module: &String,
) -> Result<(), Error> {
    sqlx::query!(
        "INSERT INTO guild_modules (guild_id, module) VALUES ($1, $2) ON CONFLICT (guild_id) DO NOTHING",
        i64::from(DbId(guild_id)),
        module,
    )
    .execute(db)
    .await?;

    Ok(())
}

async fn db_disable_module(
    db: &sqlx::PgPool,
    guild_id: Id<GuildMarker>,
    module: &String,
) -> Result<(), Error> {
    sqlx::query!(
        "DELETE FROM guild_modules WHERE guild_id = $1 AND module = $2",
        i64::from(DbId(guild_id)),
        module,
    )
    .execute(db)
    .await?;

    Ok(())
}

async fn db_guild_modules(
    db: &sqlx::PgPool,
    guild_id: Id<GuildMarker>,
) -> Result<Vec<String>, Error> {
    Ok(sqlx::query_scalar!(
        "SELECT module FROM guild_modules WHERE guild_id = $1",
        i64::from(DbId(guild_id))
    )
    .fetch_all(db)
    .await?)
}

pub(crate) async fn db_guilds_with_module(
    db: &sqlx::PgPool,
    module: &str,
) -> Result<Vec<Id<GuildMarker>>, Error> {
    Ok(sqlx::query_scalar!(
        "SELECT guild_id FROM guild_modules WHERE module = $1",
        module
    )
    .fetch_all(db)
    .await?
    .into_iter()
    .map(|id| *DbId::from(id))
    .collect())
}

pub(crate) async fn db_all_guild_modules(
    db: &sqlx::PgPool,
) -> Result<HashMap<Id<GuildMarker>, Vec<String>>, Error> {
    let rows = sqlx::query!("SELECT guild_id, module FROM guild_modules")
        .fetch_all(db)
        .await?;

    let mut result = HashMap::new();
    for row in rows {
        result
            .entry(*DbId::from(row.guild_id))
            .or_insert(Vec::new())
            .push(row.module);
    }

    Ok(result)
}
