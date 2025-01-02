CREATE TABLE guild_modules (
    guild_id BIGINT PRIMARY KEY,
    module VARCHAR(64) NOT NULL,

    UNIQUE(guild_id, module)
);
