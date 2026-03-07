-- Sales Channels
CREATE TABLE IF NOT EXISTS sales_channels (
    id          UUID         PRIMARY KEY DEFAULT uuid_generate_v4(),
    name        VARCHAR(255) NOT NULL,
    description TEXT,
    is_disabled BOOLEAN      NOT NULL DEFAULT FALSE,
    metadata    JSONB,
    created_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    deleted_at  TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS sales_channel_products (
    sales_channel_id UUID NOT NULL REFERENCES sales_channels(id) ON DELETE CASCADE,
    product_id       UUID NOT NULL REFERENCES products(id) ON DELETE CASCADE,
    PRIMARY KEY (sales_channel_id, product_id)
);

INSERT INTO sales_channels (id, name, description)
VALUES (uuid_generate_v4(), 'Default Sales Channel', 'Created by MedusaRust')
ON CONFLICT DO NOTHING;
