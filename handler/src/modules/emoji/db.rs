use sqlx::types::chrono;

use tulpje_framework::Error;
use twilight_model::id::{
    marker::{EmojiMarker, GuildMarker},
    Id,
};

use super::shared::StatsSort;
use crate::db::DbId;

#[derive(Debug)]
// TODO: tests to confirm this still matches the database structure
#[expect(dead_code, reason = "reflects database structure")]
pub(crate) struct EmojiUse {
    pub(crate) id: i64,
    pub(crate) guild_id: DbId<GuildMarker>,
    pub(crate) emoji_id: DbId<EmojiMarker>,
    pub(crate) name: String,
    pub(crate) animated: bool,
    pub(crate) created_at: chrono::NaiveDateTime,
}

#[derive(Debug, sqlx::FromRow)]
pub(crate) struct EmojiStats {
    #[sqlx(flatten)]
    pub(crate) emoji: Emoji,
    pub(crate) times_used: i64,
    pub(crate) last_used_at: chrono::NaiveDateTime,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub(crate) struct Emoji {
    #[sqlx(rename = "emoji_id")]
    pub(crate) id: DbId<EmojiMarker>,
    pub(crate) guild_id: DbId<GuildMarker>,
    pub(crate) name: String,
    pub(crate) animated: bool,
}

impl Emoji {
    pub(crate) fn new(
        id: Id<EmojiMarker>,
        guild_id: Id<GuildMarker>,
        name: String,
        animated: bool,
    ) -> Self {
        Self {
            id: DbId(id),
            guild_id: DbId(guild_id),
            name,
            animated,
        }
    }

    pub(crate) fn from_twilight(
        val: twilight_model::guild::Emoji,
        guild_id: Id<GuildMarker>,
    ) -> Self {
        Self::new(val.id, guild_id, val.name, val.animated)
    }
}

impl std::fmt::Display for Emoji {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "<{}:{}:{}>",
            if self.animated { "a" } else { "" },
            self.name,
            self.id
        )
    }
}

impl std::hash::Hash for Emoji {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl PartialEq for Emoji {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Emoji {}

pub(crate) async fn save_emoji_use(
    db: &sqlx::PgPool,
    emote: &Emoji,
    timestamp: chrono::DateTime<chrono::Utc>,
) -> Result<(), Error> {
    sqlx::query!(
        "
            INSERT INTO emoji_uses (
                guild_id,
                emoji_id,
                name,
                animated,
                created_at
            ) VALUES ($1, $2, $3, $4, $5)
        ",
        i64::from(emote.guild_id),
        i64::from(emote.id),
        emote.name,
        emote.animated,
        timestamp.naive_utc(),
    )
    .execute(db)
    .await?;

    Ok(())
}

pub(crate) async fn get_emoji_stats(
    db: &sqlx::PgPool,
    guild_id: Id<GuildMarker>,
    sort: &StatsSort,
) -> Result<Vec<EmojiStats>, Error> {
    let order_by_clause = match sort {
        StatsSort::CountDesc => "times_used DESC",
        StatsSort::CountAsc => "times_used ASC",
        StatsSort::DateDesc => "last_used_at DESC",
        StatsSort::DateAsc => "last_used_at ASC",
    };

    // NOTE: Wish we could use query_as! but we're using a dynamic SORT BY clause
    let result: Vec<EmojiStats> = sqlx::query_as(&format!(
        "
            SELECT
                emoji_id, MAX(name) as name,
                $1 AS guild_id,
                ANY_VALUE(animated) as animated,
                COUNT(emoji_id) AS times_used,
                MAX(created_at) AS last_used_at
            FROM emoji_uses
            WHERE guild_id = $1
            GROUP BY emoji_id
            ORDER BY {}
        ",
        order_by_clause
    ))
    .bind(DbId(guild_id))
    .fetch_all(db)
    .await?;

    Ok(result)
}
