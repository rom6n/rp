-- Add migration script here
-- migrate:up

CREATE TABLE refresh_tokens (
    jti TEXT NOT NULL UNIQUE,
    user_id BIGINT NOT NULL,
    token_hash TEXT NOT NULL UNIQUE,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL
);

CREATE INDEX ON refresh_tokens (user_id);
CREATE INDEX ON refresh_tokens (expires_at);
CREATE INDEX ON refresh_tokens (jti);

-- migrate:down

DROP TABLE IF EXISTS refresh_tokens;
