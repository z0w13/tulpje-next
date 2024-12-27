use tulpje_framework::Error;

#[derive(Debug)]
pub(crate) struct ModPkGuildRow {
    pub(crate) guild_id: i64,
    #[allow(dead_code)]
    pub(crate) user_id: i64,
    pub(crate) system_id: String,
    pub(crate) token: Option<String>,
}
pub(crate) async fn save_guild_settings(
    db: &sqlx::PgPool,
    guild_id: u64,
    user_id: u64,
    system_id: &String,
    token: Option<String>,
) -> Result<(), Error> {
    sqlx::query!(
        "INSERT INTO pk_guilds (guild_id, user_id, system_id, token) VALUES ($1, $2, $3, $4) ON CONFLICT (guild_id) DO UPDATE SET system_id = $3, token = $4",
        i64::try_from(guild_id)?,
        i64::try_from(user_id)?,
        system_id,
        token,
    )
    .execute(db)
    .await?;

    Ok(())
}

pub(crate) async fn get_guild_settings_for_id(
    db: &sqlx::PgPool,
    guild_id: u64,
) -> Result<Option<ModPkGuildRow>, Error> {
    Ok(sqlx::query_as!(
        ModPkGuildRow,
        "SELECT guild_id, user_id, system_id, token FROM pk_guilds WHERE guild_id = $1",
        i64::try_from(guild_id)?
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
