use twilight_model::{
    application::interaction::application_command::CommandOptionValue,
    channel::message::{
        component::{ActionRow, SelectMenu, SelectMenuType},
        Embed,
    },
    guild::Guild,
    http::interaction::{InteractionResponse, InteractionResponseType},
};
use twilight_util::builder::{embed::EmbedBuilder, InteractionResponseDataBuilder};

use tulpje_framework::Error;

use super::db;
use crate::{
    context::{CommandContext, ComponentInteractionContext},
    modules::emoji::shared::StatsSort,
};

fn create_emoji_stats_sort_menu() -> SelectMenu {
    SelectMenu {
        custom_id: "emoji_stats_sort".into(),
        kind: SelectMenuType::Text,
        options: Some(vec![
            StatsSort::CountDesc.into(),
            StatsSort::CountAsc.into(),
            StatsSort::DateDesc.into(),
            StatsSort::DateAsc.into(),
        ]),
        placeholder: Some("Sort".into()),

        // defaults
        disabled: false,
        max_values: None,
        min_values: None,
        default_values: None,
        channel_types: None,
    }
}

async fn create_emoji_stats_embed(
    db: &sqlx::PgPool,
    guild: &Guild,
    sort: &StatsSort,
) -> Result<Embed, Error> {
    let emoji_stats = db::get_emoji_stats(db, guild.id, sort).await?;
    let emoji_str = if !emoji_stats.is_empty() {
        emoji_stats
            .into_iter()
            .map(|emoji_stats| {
                format!(
                    "{} • Used {} times • Last used <t:{}:R>",
                    emoji_stats.emoji,
                    emoji_stats.times_used,
                    emoji_stats.last_used_at.and_utc().timestamp(),
                )
            })
            .collect::<Vec<String>>()
            .join("\n")
    } else {
        "No Data".to_string()
    };

    Ok(EmbedBuilder::new()
        .title(format!("{} Emotes in {}", sort.name(), guild.name))
        .description(emoji_str)
        .build())
}

pub async fn handle_emoji_stats_sort(ctx: ComponentInteractionContext) -> Result<(), Error> {
    if ctx.interaction.custom_id != "emoji_stats_sort" {
        tracing::debug!(
            "ignoring interaction with incorrect custom_id: {}",
            ctx.interaction.custom_id
        );
        return Ok(());
    }
    tracing::trace!(interaction = ?ctx.interaction);

    ctx.response(InteractionResponse {
        kind: InteractionResponseType::DeferredUpdateMessage,
        data: None,
    })
    .await?;

    let Some(sort_by) = ctx.interaction.values.first() else {
        return Err("couldn't get selected value".into());
    };
    tracing::trace!(?sort_by);

    let sort = StatsSort::try_from_string(sort_by)?;
    tracing::trace!(sort = ?sort);

    let guild = ctx.guild().await?.ok_or("outside of guild")?;

    if let Err(err) = ctx
        .interaction()
        .update_response(&ctx.event.token)
        .embeds(Some(&[create_emoji_stats_embed(
            &ctx.services.db,
            &guild,
            &sort,
        )
        .await?]))
        .components(Some(&[ActionRow {
            components: vec![create_emoji_stats_sort_menu().into()],
        }
        .into()]))
        .await
    {
        tracing::warn!(?err, "failed to update message");
    }

    Ok(())
}

pub async fn cmd_emoji_stats(ctx: CommandContext) -> Result<(), Error> {
    tracing::info!(command_info = ?ctx.command.options);

    let sort = if let Some(option) = ctx.command.options.first() {
        if let CommandOptionValue::String(str) = &option.value {
            StatsSort::try_from_string(str)?
        } else {
            StatsSort::CountDesc
        }
    } else {
        StatsSort::CountDesc
    };

    let guild = ctx.guild().await?.ok_or("not in guild")?;

    let response = InteractionResponse {
        kind: InteractionResponseType::ChannelMessageWithSource,
        data: Some(
            InteractionResponseDataBuilder::new()
                .embeds([create_emoji_stats_embed(&ctx.services.db, &guild, &sort).await?])
                .components([ActionRow {
                    components: vec![create_emoji_stats_sort_menu().into()],
                }
                .into()])
                .build(),
        ),
    };

    ctx.response(response).await?;

    Ok(())
}
