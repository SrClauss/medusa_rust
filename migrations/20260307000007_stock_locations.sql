-- Stock Locations
CREATE TABLE IF NOT EXISTS stock_locations (
    id          UUID         PRIMARY KEY DEFAULT uuid_generate_v4(),
    name        VARCHAR(255) NOT NULL,
    address     JSONB,
    metadata    JSONB,
    created_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    deleted_at  TIMESTAMPTZ
);

-- Update inventory_levels to reference stock_locations properly
-- (location_id was already UUID, now we can enforce FK if needed)

-- Stock location ↔ fulfillment provider join
CREATE TABLE IF NOT EXISTS stock_location_fulfillment_providers (
    stock_location_id UUID         NOT NULL REFERENCES stock_locations(id) ON DELETE CASCADE,
    provider_id       VARCHAR(255) NOT NULL REFERENCES fulfillment_providers(id) ON DELETE CASCADE,
    PRIMARY KEY (stock_location_id, provider_id)
);

-- Stock location ↔ sales channel join
CREATE TABLE IF NOT EXISTS stock_location_sales_channels (
    stock_location_id UUID NOT NULL REFERENCES stock_locations(id) ON DELETE CASCADE,
    sales_channel_id  UUID NOT NULL REFERENCES sales_channels(id) ON DELETE CASCADE,
    PRIMARY KEY (stock_location_id, sales_channel_id)
);

-- Fulfillment sets (used for service zone configuration)
CREATE TABLE IF NOT EXISTS fulfillment_sets (
    id         UUID         PRIMARY KEY DEFAULT uuid_generate_v4(),
    name       VARCHAR(255) NOT NULL,
    type       VARCHAR(50)  NOT NULL DEFAULT 'shipping',
    metadata   JSONB,
    created_at TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS stock_location_fulfillment_sets (
    stock_location_id UUID NOT NULL REFERENCES stock_locations(id) ON DELETE CASCADE,
    fulfillment_set_id UUID NOT NULL REFERENCES fulfillment_sets(id) ON DELETE CASCADE,
    PRIMARY KEY (stock_location_id, fulfillment_set_id)
);
