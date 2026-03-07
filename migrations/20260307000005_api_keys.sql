-- API Keys
CREATE TABLE IF NOT EXISTS api_keys (
    id          UUID         PRIMARY KEY DEFAULT uuid_generate_v4(),
    token       VARCHAR(255) NOT NULL UNIQUE,
    title       VARCHAR(255) NOT NULL,
    type        VARCHAR(50)  NOT NULL DEFAULT 'secret',
    last_used_at TIMESTAMPTZ,
    created_by  UUID,
    revoked_by  UUID,
    revoked_at  TIMESTAMPTZ,
    created_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

-- Sales channel ↔ API key join
CREATE TABLE IF NOT EXISTS api_key_sales_channels (
    api_key_id       UUID NOT NULL REFERENCES api_keys(id) ON DELETE CASCADE,
    sales_channel_id UUID NOT NULL REFERENCES sales_channels(id) ON DELETE CASCADE,
    PRIMARY KEY (api_key_id, sales_channel_id)
);
