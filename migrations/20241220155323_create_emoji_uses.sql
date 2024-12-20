-- TODO: Check if we error on overflowing 9 223 372 036 854 775 807
CREATE TABLE emoji_uses (
    id SERIAL PRIMARY KEY,
    guild_id BIGINT NOT NULL,
    emoji_id BIGINT NOT NULL,
    name VARCHAR(32) NOT NULL,
    animated BOOL NOT NULL DEFAULT false,
    created_at TIMESTAMP NOT NULL
);
