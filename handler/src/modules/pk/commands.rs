use pkrs::model::PkId;
use tracing::debug;

use tulpje_framework::Error;

use super::db;
use crate::context::CommandContext;

// TODO: command to see current settings

pub async fn setup_pk(ctx: CommandContext) -> Result<(), Error> {
    let Some(guild) = ctx.guild().await? else {
        unreachable!("command is guild_only");
    };

    ctx.defer_ephemeral().await?;

    let user_id = ctx.event.author_id().ok_or("no author?")?;
    let system_id = ctx.get_arg_string("system_id")?;
    let token = ctx.get_arg_string_optional("token")?;

    debug!(
        guild_id = guild.id.get(),
        guild_name = guild.name,
        command = "setup-pk",
        system_id = system_id
    );

    // sanitise and validate system id
    let system_id = system_id.trim().replace("-", "").to_lowercase();
    if !system_id.chars().all(|c| char::is_ascii_alphabetic(&c)) {
        ctx.update(format!("error: invalid system id, {}", system_id))
            .await?;
        return Ok(());
    }

    db::save_guild_settings(
        &ctx.services.db,
        guild.id,
        user_id,
        &system_id,
        token.clone(),
    )
    .await?;

    let pk = pkrs::client::PkClient {
        token: token.unwrap_or_default(),
        ..Default::default()
    };

    // TODO: fix pkrs to actually handle 404s correctly
    let system = match pk.get_system(&PkId(system_id.clone())).await {
        Ok(system) => system,
        Err(err) => {
            ctx.update(format!(
                "PluralKit API is having issues or system doesn't exist: {:?}",
                err
            ))
            .await?;
            return Ok(());
        }
    };

    // Inform user of success
    let response_text = format!(
        "PluralKit module setup with system: {}",
        match system.name {
            Some(system_name) => format!("{} (`{}`)", system_name, system_id),
            None => format!("`{}`", system_id),
        }
    );

    ctx.update(response_text).await?;

    Ok(())
}
