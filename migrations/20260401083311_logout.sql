CREATE TABLE IF NOT EXISTS token_blocklist (
    jti        TEXT        PRIMARY KEY,
    expires_at TIMESTAMPTZ NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_blocklist_expires_at ON token_blocklist (expires_at);
