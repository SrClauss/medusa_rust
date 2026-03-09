-- OAuth users join table
-- Links a local `users` row to one or more social login provider identities.
-- Mirrors MedusaJS strategy pattern: provider + provider_user_id → user.

CREATE TABLE IF NOT EXISTS oauth_users (
    id                  UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id             UUID         NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    provider            TEXT         NOT NULL,  -- e.g. "google", "facebook", "github"
    provider_user_id    TEXT         NOT NULL,  -- ID assigned by the provider
    profile_data        JSONB        NOT NULL DEFAULT '{}',
    created_at          TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    UNIQUE (provider, provider_user_id)
);

CREATE INDEX IF NOT EXISTS idx_oauth_users_user_id  ON oauth_users (user_id);
CREATE INDEX IF NOT EXISTS idx_oauth_users_provider ON oauth_users (provider);
