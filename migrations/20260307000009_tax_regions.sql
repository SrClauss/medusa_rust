-- Tax Regions
CREATE TABLE IF NOT EXISTS tax_regions (
    id                UUID         PRIMARY KEY DEFAULT uuid_generate_v4(),
    provider_id       VARCHAR(255),
    country_code      VARCHAR(2)   NOT NULL,
    province_code     VARCHAR(50),
    parent_id         UUID         REFERENCES tax_regions(id),
    metadata          JSONB,
    created_at        TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at        TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    deleted_at        TIMESTAMPTZ
);

-- Tax overrides (rules per region)
CREATE TABLE IF NOT EXISTS tax_overrides (
    id            UUID         PRIMARY KEY DEFAULT uuid_generate_v4(),
    tax_region_id UUID         NOT NULL REFERENCES tax_regions(id) ON DELETE CASCADE,
    reference_id  UUID,
    reference     VARCHAR(50),
    name          VARCHAR(255) NOT NULL,
    code          VARCHAR(255),
    rate          FLOAT        NOT NULL DEFAULT 0,
    metadata      JSONB,
    created_at    TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);
