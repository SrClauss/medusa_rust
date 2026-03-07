-- Invites
CREATE TABLE IF NOT EXISTS invites (
    id         UUID         PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_email VARCHAR(255) NOT NULL UNIQUE,
    role       VARCHAR(50)  NOT NULL DEFAULT 'member',
    accepted   BOOLEAN      NOT NULL DEFAULT FALSE,
    token      VARCHAR(255) NOT NULL UNIQUE,
    expires_at TIMESTAMPTZ,
    metadata   JSONB,
    created_at TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);
