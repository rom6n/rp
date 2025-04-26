-- Add migration script here
-- migrate:up

CREATE TABLE IF NOT EXISTS users (
    id BIGSERIAL PRIMARY KEY,
    nickname TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    password TEXT NOT NULL
);



-- migrate:down
DROP TABLE IF EXISTS users;