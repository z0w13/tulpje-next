use twilight_model::id::{
    marker::{GuildMarker, UserMarker},
    Id,
};

use tulpje_framework::Error;

use crate::db::DbId;

#[derive(Debug)]
// TODO: tests to confirm this still matches the database structure
#[expect(dead_code, reason = "reflects database structure")]
pub(crate) struct ModPkGuildRow {
    pub(crate) guild_id: DbId<GuildMarker>,
    pub(crate) user_id: DbId<UserMarker>,
    pub(crate) system_id: String,
    pub(crate) token: Option<String>,
}
pub(crate) async fn save_guild_settings(
    db: &sqlx::PgPool,
    guild_id: Id<GuildMarker>,
    user_id: Id<UserMarker>,
    system_id: &String,
    token: Option<String>,
) -> Result<(), Error> {
    sqlx::query!(
        "INSERT INTO pk_guilds (guild_id, user_id, system_id, token) VALUES ($1, $2, $3, $4) ON CONFLICT (guild_id) DO UPDATE SET system_id = $3, token = $4",
        i64::from(DbId(guild_id)),
        i64::from(DbId(user_id)),
        system_id,
        token,
    )
    .execute(db)
    .await?;

    Ok(())
}

pub(crate) async fn get_guild_settings_for_id(
    db: &sqlx::PgPool,
    guild_id: Id<GuildMarker>,
) -> Result<Option<ModPkGuildRow>, Error> {
    Ok(sqlx::query_as!(
        ModPkGuildRow,
        "SELECT guild_id, user_id, system_id, token FROM pk_guilds WHERE guild_id = $1",
        i64::from(DbId(guild_id))
    )
    .fetch_optional(db)
    .await?)
}

pub(crate) async fn get_guild_settings(db: &sqlx::PgPool) -> Result<Vec<ModPkGuildRow>, Error> {
    Ok(sqlx::query_as!(
        ModPkGuildRow,
        "SELECT guild_id, user_id, system_id, token FROM pk_guilds",
    )
    .fetch_all(db)
    .await?)
}
