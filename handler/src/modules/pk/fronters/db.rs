use tulpje_framework::Error;

pub(crate) struct ModPkFrontersRow {
    pub(crate) guild_id: u64,
    pub(crate) category_id: u64,
}

pub(crate) async fn get_fronter_categories(
    db: &sqlx::PgPool,
) -> Result<Vec<ModPkFrontersRow>, Error> {
    let result = sqlx::query!("SELECT guild_id, category_id FROM pk_fronters")
        .fetch_all(db)
        .await?;

    // TODO: Better handling of try_into()?
    //       I mean, we should actually test what happens when surpassing i64::MAX and such
    Ok(result
        .into_iter()
        .map(|row| ModPkFrontersRow {
            guild_id: row.guild_id.try_into().unwrap(),
            category_id: row.category_id.try_into().unwrap(),
        })
        .collect())
}

pub(crate) async fn get_fronter_category(
    db: &sqlx::PgPool,
    guild_id: u64,
) -> Result<Option<u64>, Error> {
    let result = sqlx::query_scalar!(
        "SELECT category_id FROM pk_fronters WHERE guild_id = $1",
        i64::try_from(guild_id)?,
    )
    .fetch_optional(db)
    .await?;

    match result {
        Some(cat_id) => Ok(Some(cat_id.try_into()?)),
        None => Ok(None),
    }
}

pub(crate) async fn save_fronter_category(
    db: &sqlx::PgPool,
    guild_id: u64,
    channel_id: u64,
) -> Result<(), Error> {
    sqlx::query!(
        "INSERT INTO pk_fronters (guild_id, category_id) VALUES ($1, $2) ON CONFLICT (guild_id) DO UPDATE SET category_id = $2",
        i64::try_from(guild_id)?,
        i64::try_from(channel_id)?,
    )
    .execute(db)
    .await?;

    Ok(())
}

#[expect(
    dead_code,
    reason = "this isn't used anywhere yet but is a useful utility function nonetheless"
)]
pub(crate) async fn get_system_count(db: &sqlx::PgPool) -> Result<usize, Error> {
    let system_count = sqlx::query_scalar!("SELECT COUNT(DISTINCT system_id) FROM pk_fronters INNER JOIN pk_guilds ON pk_fronters.guild_id = pk_guilds.guild_id")
        .fetch_one(db)
        .await?;

    Ok(system_count.unwrap_or(0) as usize)
}
