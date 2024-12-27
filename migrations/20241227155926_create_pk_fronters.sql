-- TODO: Check if we error on overflowing 9 223 372 036 854 775 807
CREATE TABLE pk_fronters (
    guild_id BIGINT PRIMARY KEY,
    category_id BIGINT NOT NULL,

    UNIQUE(guild_id, category_id)
);
