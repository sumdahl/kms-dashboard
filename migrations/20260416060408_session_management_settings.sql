-- Session management admin configurable settings
CREATE TABLE IF NOT EXISTS app_settings (
    key VARCHAR(100) PRIMARY KEY,
    value TEXT NOT NULL,
    description TEXT,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_by UUID REFERENCES users(user_id) ON DELETE SET NULL
);

CREATE INDEX IF NOT EXISTS idx_app_settings_key ON app_settings(key);

-- User sessions table for hybrid JWT + session auth strategy
CREATE TABLE IF NOT EXISTS user_sessions (
    session_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(user_id) ON DELETE CASCADE,
    refresh_token TEXT UNIQUE,
    ip_address INET,
    user_agent TEXT,
    data JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL,
    last_activity TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_user_sessions_user_id ON user_sessions(user_id);
CREATE INDEX IF NOT EXISTS idx_user_sessions_expires_at ON user_sessions(expires_at);
CREATE INDEX IF NOT EXISTS idx_user_sessions_refresh_token ON user_sessions(refresh_token);

-- Insert default settings
INSERT INTO app_settings (key, value, description) VALUES
    ('auth_strategy', 'hybrid', 'Authentication strategy: jwt, session, hybrid'),
    ('jwt_access_ttl_minutes', '15', 'JWT access token TTL in minutes (5-1440)'),
    ('session_refresh_ttl_hours', '168', 'Session/refresh token TTL in hours (1-720)'),
    ('max_concurrent_sessions', '5', 'Maximum concurrent sessions per user'),
    ('logout_on_browser_close', 'false', 'Logout automatically on browser close'),
    ('force_logout_on_password_change', 'true', 'Force logout all sessions on password change'),
    ('ip_restriction_enabled', 'false', 'Enable IP address based session restrictions'),
    ('remember_me_extension_hours', '72', 'Remember me session extension in hours')
ON CONFLICT (key) DO NOTHING;
