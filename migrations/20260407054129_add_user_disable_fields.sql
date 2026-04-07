ALTER TABLE users
    ADD COLUMN is_active        BOOLEAN     NOT NULL DEFAULT TRUE,
    ADD COLUMN session_version  INT         NOT NULL DEFAULT 0,
    ADD COLUMN disabled_at      TIMESTAMPTZ,
    ADD COLUMN disabled_by      UUID REFERENCES users(user_id),
    ADD COLUMN disabled_reason  TEXT;

CREATE INDEX idx_users_is_active ON users(is_active);

CREATE TABLE user_audit_log (
    id             UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    target_user_id UUID        NOT NULL REFERENCES users(user_id),
    actor_id       UUID        NOT NULL REFERENCES users(user_id),
    action         TEXT        NOT NULL,
    reason         TEXT,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_audit_log_target ON user_audit_log(target_user_id);
CREATE INDEX idx_audit_log_actor  ON user_audit_log(actor_id);
