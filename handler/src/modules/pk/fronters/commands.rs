use std::collections::{HashMap, HashSet};

use pkrs::model::PkId;
use serde_either::StringOrStruct;
use tracing::error;
use twilight_http::Client;
use twilight_model::channel::permission_overwrite::{PermissionOverwrite, PermissionOverwriteType};
use twilight_model::channel::{Channel, ChannelType};
use twilight_model::guild::{Guild, Permissions};
use twilight_model::id::marker::{ChannelMarker, GuildMarker};
use twilight_model::id::Id;

use tulpje_framework::Error;

use super::super::util::get_member_name;
use super::db;
use crate::context::CommandContext;
use crate::modules::pk::db::{get_guild_settings_for_id, ModPkGuildRow};

async fn get_desired_fronters(system: &PkId, token: String) -> Result<HashSet<String>, Error> {
    let pk = pkrs::client::PkClient {
        token,
        ..Default::default()
    };

    let fronters = pk
        .get_system_fronters(system)
        .await?
        .members
        .into_iter()
        .filter_map(|m| match m {
            StringOrStruct::String(_) => None,
            StringOrStruct::Struct(member) => Some(get_member_name(&member)),
        })
        .collect();

    Ok(fronters)
}

async fn get_fronter_channels(
    client: &Client,
    guild: Id<GuildMarker>,
    cat_id: Id<ChannelMarker>,
) -> Result<Vec<Channel>, Error> {
    let channels = client
        .guild_channels(guild)
        .await?
        .models()
        .await?
        .into_iter()
        .filter(|c| c.parent_id.is_some() && c.parent_id.unwrap() == cat_id)
        .collect();

    Ok(channels)
}

async fn get_fronter_category(
    client: &Client,
    guild: &Guild,
    opt_cat_name: Option<String>,
) -> Result<Option<Channel>, Error> {
    let cat_name = opt_cat_name
        .unwrap_or_else(|| "current fronters".into())
        .to_lowercase();

    Ok(client
        .guild_channels(guild.id)
        .await?
        .models()
        .await?
        .into_iter()
        .find(|c| {
            c.name
                .clone()
                .expect("guild channels have names")
                .to_lowercase()
                == cat_name
                && c.kind == ChannelType::GuildCategory
        }))
}

// TODO: Instrument why this bitch slow, are we even using discord's cache?
//       should definitely do that
pub(crate) async fn update_fronter_channels(
    client: &Client,
    guild: Guild,
    gs: &ModPkGuildRow,
    cat: Channel,
) -> Result<(), Error> {
    let fronter_channels = get_fronter_channels(client, guild.id, cat.id).await?;
    let desired_fronters = get_desired_fronters(
        &PkId(gs.system_id.clone()),
        gs.token.clone().unwrap_or_default(),
    )
    .await?;
    let current_fronters: HashSet<String> = fronter_channels
        .iter()
        .map(|c| c.name.clone().expect("guild channels have names"))
        .collect();

    let mut fronter_channel_map: HashMap<String, Channel> = fronter_channels
        .iter()
        .map(|c| {
            (
                c.name.clone().expect("guild channels have names"),
                c.to_owned(),
            )
        })
        .collect();

    let fronter_pos_map: HashMap<String, u16> = desired_fronters
        .iter()
        .enumerate()
        // WARN: could this result in a panic/error? usize into u16
        .map(|(k, v)| (v.to_owned(), k.try_into().unwrap()))
        .collect();

    let delete_fronters = current_fronters.difference(&desired_fronters);
    let create_fronters = desired_fronters.difference(&current_fronters);

    for fronter in delete_fronters {
        #[expect(
            clippy::indexing_slicing,
            reason = "`delete_fronters` should only contain keys from `fronter_channel_map`"
        )]
        let channel = &fronter_channel_map[fronter];
        if let Err(e) = client.delete_channel(channel.id).await {
            error!("error deleting channel '{}': {}", fronter, e);
            continue;
        }

        fronter_channel_map.remove(fronter);
    }

    for fronter in create_fronters {
        let pos = fronter_pos_map
            .get(fronter)
            .expect("couldn't get position for fronter, this should never happen!");

        let permissions = vec![PermissionOverwrite {
            deny: Permissions::CONNECT,
            allow: Permissions::empty(),
            id: guild.id.cast(),
            kind: PermissionOverwriteType::Role,
        }];

        let channel = match client
            .create_guild_channel(guild.id, fronter)
            .permission_overwrites(&permissions)
            .position(u64::from(*pos))
            .parent_id(cat.id)
            .kind(ChannelType::GuildVoice)
            .await
        {
            Ok(response) => match response.model().await {
                Ok(chan) => chan,
                Err(e) => {
                    error!("error deserialising fronter channel '{}': {}", fronter, e);
                    continue;
                }
            },
            Err(e) => {
                error!("error creating fronter channel '{}': {}", fronter, e);
                continue;
            }
        };

        fronter_channel_map.insert(fronter.to_owned(), channel.clone());
    }
    for (name, position) in fronter_pos_map.iter() {
        let channel = fronter_channel_map
            .get(name)
            .expect("couldn't get channel from fronter_channel_map, this should never happen!")
            .to_owned();

        if channel.position.is_some_and(|p| p == i32::from(*position)) {
            continue;
        }

        if let Err(e) = client
            .update_channel(channel.id)
            .position(u64::from(*position))
            .await
        {
            error!("error moving channel '{}': {}", name, e);
            continue;
        }
    }

    Ok(())
}

pub(crate) async fn update_fronters(ctx: CommandContext) -> Result<(), Error> {
    let Some(guild) = ctx.guild().await? else {
        unreachable!("command is guild_only");
    };

    ctx.defer_ephemeral().await?;

    let Some(cat_id) = db::get_fronter_category(&ctx.services.db, guild.id).await? else {
        ctx.update("fronter category not set-up, please run /setup-fronters")
            .await?;
        return Ok(());
    };

    let Some(gs) = get_guild_settings_for_id(&ctx.services.db, guild.id).await? else {
        ctx.update("PluralKit module not set-up, please run /setup-pk")
            .await?;
        return Ok(());
    };

    let cat = ctx
        .client()
        .channel(Id::<ChannelMarker>::new(cat_id))
        .await?
        .model()
        .await?;

    cat.guild_id
        .ok_or_else(|| format!("channel {} isn't a guild channel", cat_id))?;

    update_fronter_channels(&ctx.client(), guild, &gs, cat).await?;

    ctx.update("fronter list updated!").await?;
    Ok(())
}

async fn create_or_get_fronter_channel(
    client: &Client,
    guild: &Guild,
    cat_name: String,
) -> Result<Channel, Error> {
    if let Some(fronters_category) =
        get_fronter_category(client, guild, Some(cat_name.clone())).await?
    {
        return Ok(fronters_category);
    }

    let permissions = vec![PermissionOverwrite {
        deny: Permissions::VIEW_CHANNEL,
        allow: Permissions::empty(),
        id: guild.id.cast(),
        kind: PermissionOverwriteType::Role,
    }];

    Ok(client
        .create_guild_channel(guild.id, &cat_name)
        .permission_overwrites(&permissions)
        .kind(ChannelType::GuildCategory)
        .await?
        .model()
        .await?)
}

pub(crate) async fn setup_fronters(ctx: CommandContext) -> Result<(), Error> {
    let Some(guild) = ctx.guild().await? else {
        unreachable!("command is guild_only");
    };

    ctx.defer_ephemeral().await?;

    let name = ctx.get_arg_string("name")?;
    let fronters_category = create_or_get_fronter_channel(&ctx.client, &guild, name).await?;

    // Save category into db
    db::save_fronter_category(&ctx.services.db, guild.id, fronters_category.id).await?;

    // Inform user of success
    ctx.update("fronter list setup!").await?;
    Ok(())
}
