use base64::{prelude::BASE64_STANDARD, Engine as _};
use futures_util::StreamExt;
use twilight_http::Client;

use tulpje_framework::Error;
use twilight_model::id::marker::{EmojiMarker, GuildMarker};
use twilight_model::id::Id;

use crate::context::CommandContext;
use crate::modules::emoji::db::Emoji;
use crate::modules::emoji::shared::parse_emojis_from_string;

// requires CREATE_GUILD_EXPRESSIONS permission
pub(crate) async fn command(ctx: CommandContext) -> Result<(), Error> {
    let Some(guild) = ctx.guild().await? else {
        unreachable!("command is guild_only");
    };

    let emojis = parse_emojis_from_string(
        Id::<GuildMarker>::new(1), /* DUMMY */
        &ctx.get_arg_string("emoji")?,
    );
    if emojis.is_empty() {
        ctx.reply("no emojis found").await?;
        return Ok(());
    }

    if let Some(new_name) = ctx.get_arg_string_optional("new_name")? {
        // add single emote with new_name
        if emojis.len() > 1 {
            ctx.reply("can't add more than one emote at a time when specifying name")
                .await?;
            return Ok(());
        }

        // defer, we might be a while
        ctx.defer().await?;

        ctx.update(
            match clone_emoji(&ctx.client, guild.id, emojis.first().unwrap(), &new_name).await {
                Ok(emoji) => format!("**Added:** {}", emoji),
                Err(err) => format!("**Error:** {}", err),
            },
        )
        .await?;

        return Ok(());
    } else if emojis.len() > 10 {
        ctx.reply("**ERROR:** can't add more than 10 emotes at once")
            .await?;
        return Ok(());
    } else {
        // add multiple emotes
        let prefix = ctx.get_arg_string_optional("prefix")?.unwrap_or("".into());

        // defer, we might be a while
        ctx.defer().await?;

        // what a fucken mess to have async map, but it works :)
        let emoji_results: Vec<Result<Emoji, EmojiError>> =
            futures_util::stream::iter(
                emojis.into_iter().map(|e| {
                    let prefix = prefix.clone();
                    let client = ctx.client.clone();

                    async move {
                        clone_emoji(&client, guild.id, &e, &format!("{}{}", prefix, e.name)).await
                    }
                }),
            )
            .buffered(1)
            .collect()
            .await;

        let emojis_added: Vec<String> = emoji_results
            .iter()
            .filter_map(|r| match r {
                Ok(emoji) => Some(emoji.to_string()),
                Err(_) => None,
            })
            .collect();

        let emoji_errors: Vec<String> = emoji_results
            .iter()
            .filter_map(|r| match r {
                Ok(_) => None,
                Err(e) => Some(format!("* {}", e)),
            })
            .collect();

        let reply = format!(
            "{}\n{}",
            match emojis_added.is_empty() {
                true => "".into(),
                false => format!("**Added:** {}", emojis_added.join("")),
            },
            match emoji_errors.is_empty() {
                true => "".into(),
                false => format!("**Errors:**\n{}", emoji_errors.join("\n")),
            },
        );

        if let Err(err) = ctx.update(&reply).await {
            tracing::warn!(?err, "failed to respond to command")
        }
    }

    Ok(())
}

async fn download_emoji(id: Id<EmojiMarker>, animated: bool) -> Result<String, reqwest::Error> {
    reqwest::get(format!(
        "https://cdn.discordapp.com/emojis/{}.{}",
        id,
        match animated {
            true => "gif",
            false => "webp",
        },
    ))
    .await?
    .error_for_status()? // error if we don't get a 200 status
    .bytes()
    .await
    .map(|b| {
        // convert to a data uri
        format!(
            "data:{};base64,{}",
            match animated {
                true => "image/gif",
                false => "image/webp",
            },
            BASE64_STANDARD.encode(b),
        )
    })
}

async fn clone_emoji(
    client: &Client,
    guild_id: Id<GuildMarker>,
    emoji: &Emoji,
    new_name: &str,
) -> Result<Emoji, EmojiError> {
    let emoji_data_uri = download_emoji(*emoji.id, emoji.animated)
        .await
        .map_err(|err| EmojiError::Download(emoji.clone(), err))?;

    let new_emoji = client
        .create_emoji(guild_id, new_name, &emoji_data_uri)
        .await
        .map_err(|e| EmojiError::Create(emoji.clone(), e))?
        .model()
        .await
        .map_err(|e| EmojiError::Other(emoji.clone(), e.into()))?;

    Ok(Emoji::from_twilight(new_emoji, guild_id))
}

pub(crate) enum EmojiError {
    Other(Emoji, Error),
    Download(Emoji, reqwest::Error),
    Create(Emoji, twilight_http::Error),
}

impl EmojiError {
    pub(crate) fn as_str(&self) -> String {
        match self {
            Self::Download(emoji, err) => {
                format!("error downloading emoji ({}): {}", emoji.name, err)
            }
            Self::Create(emoji, err) => {
                format!("error creating emoji ({}): {}", emoji.name, err)
            }
            Self::Other(emoji, err) => format!("unkown error ({}): {}", emoji.name, err),
        }
    }
}

impl std::fmt::Display for EmojiError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt.write_str(&self.as_str())
    }
}
impl std::fmt::Debug for EmojiError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self, fmt)
    }
}

impl std::error::Error for EmojiError {}
