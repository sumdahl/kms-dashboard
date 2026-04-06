CREATE TABLE token_blocklist (
    jti        TEXT        PRIMARY KEY,
    expires_at TIMESTAMPTZ NOT NULL
);

CREATE INDEX idx_blocklist_expires_at ON token_blocklist (expires_at);
