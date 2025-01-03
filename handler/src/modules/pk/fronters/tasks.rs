use tracing::{error, info, warn};
use twilight_http::Client;

use tulpje_framework::Error;

use self::pk::db::ModPkGuildRow;
use super::db::ModPkFrontersRow;
use crate::{
    context::TaskContext,
    modules::{core, pk},
};

pub(crate) async fn update_fronters(ctx: TaskContext) -> Result<(), Error> {
    let fronter_cats = super::db::get_fronter_categories(&ctx.services.db).await?;
    let guild_settings = pk::db::get_guild_settings(&ctx.services.db).await?;
    let pk_guilds = core::db_guilds_with_module(&ctx.services.db, "pluralkit").await?;

    for cat in fronter_cats {
        if !pk_guilds.contains(&cat.guild_id) {
            tracing::debug!(
                "skipping guild {}, it doesn't have the pluralkit module enabled",
                cat.guild_id
            );
            continue;
        }

        let cur_guild_settings = guild_settings.iter().find(|gs| gs.guild_id == cat.guild_id);

        if let Some(gs) = cur_guild_settings {
            if let Err(err) = update_fronters_for_guild(&ctx.client, gs, &cat).await {
                error!(
                    guild_id = ?cat.guild_id,
                    category_id = ?cat.category_id,
                    err
                );
            }
        } else {
            warn!(
                guild_id = ?cat.guild_id,
                "couldn't find guild settings for guild"
            );
        }
    }

    Ok(())
}

async fn update_fronters_for_guild(
    client: &Client,
    gs: &ModPkGuildRow,
    cat: &ModPkFrontersRow,
) -> Result<(), Error> {
    let guild = client.guild(cat.guild_id.0).await?.model().await?;

    let cat = client
        .channel(cat.category_id.0)
        .await
        .map_err(|err| {
            format!(
                "couldn't find category for guild '{}' ({}) {}",
                guild.name, guild.id, err
            )
        })?
        .model()
        .await?;

    cat.guild_id.ok_or(format!(
        "channel {} for guild '{}' ({}) isn't a guild channel",
        cat.id, guild.name, guild.id
    ))?;

    super::commands::update_fronter_channels(client, guild.clone(), gs, cat)
        .await
        .map_err(|err| {
            format!(
                "error updating fronters for {} ({}): {}",
                guild.name, guild.id, err
            )
        })?;

    info!(
        guild.id = guild.id.get(),
        guild.name = guild.name,
        "fronters updated"
    );

    Ok(())
}
