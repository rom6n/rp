-- Add migration script here
DROP INDEX IF EXISTS refresh_tokens_jti_idx;
DROP INDEX IF EXISTS refresh_tokens_user_id_idx;
DROP INDEX IF EXISTS refresh_tokens_expires_at_idx;

DROP TABLE IF EXISTS refresh_tokens;