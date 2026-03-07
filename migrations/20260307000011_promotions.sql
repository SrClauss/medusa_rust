-- Promotions (Medusa v2 replaces Discounts for coupon-style campaigns)
CREATE TABLE IF NOT EXISTS promotions (
    id             UUID         PRIMARY KEY DEFAULT uuid_generate_v4(),
    code           VARCHAR(255) NOT NULL UNIQUE,
    is_automatic   BOOLEAN      NOT NULL DEFAULT FALSE,
    type           VARCHAR(50)  NOT NULL DEFAULT 'standard',
    status         VARCHAR(50)  NOT NULL DEFAULT 'draft',
    campaign_id    UUID,
    metadata       JSONB,
    created_at     TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at     TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    deleted_at     TIMESTAMPTZ
);

-- Campaigns
CREATE TABLE IF NOT EXISTS campaigns (
    id              UUID         PRIMARY KEY DEFAULT uuid_generate_v4(),
    name            VARCHAR(255) NOT NULL,
    description     TEXT,
    identifier      VARCHAR(255) NOT NULL UNIQUE,
    starts_at       TIMESTAMPTZ,
    ends_at         TIMESTAMPTZ,
    metadata        JSONB,
    created_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    deleted_at      TIMESTAMPTZ
);

ALTER TABLE promotions
    ADD CONSTRAINT fk_promotions_campaign
    FOREIGN KEY (campaign_id) REFERENCES campaigns(id)
    DEFERRABLE INITIALLY DEFERRED;

-- Promotion rules
CREATE TABLE IF NOT EXISTS promotion_rules (
    id             UUID         PRIMARY KEY DEFAULT uuid_generate_v4(),
    promotion_id   UUID         NOT NULL REFERENCES promotions(id) ON DELETE CASCADE,
    attribute      VARCHAR(255) NOT NULL,
    operator       VARCHAR(50)  NOT NULL DEFAULT 'eq',
    description    TEXT,
    metadata       JSONB,
    created_at     TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at     TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

-- Promotion rule values
CREATE TABLE IF NOT EXISTS promotion_rule_values (
    id                 UUID         PRIMARY KEY DEFAULT uuid_generate_v4(),
    promotion_rule_id  UUID         NOT NULL REFERENCES promotion_rules(id) ON DELETE CASCADE,
    value              VARCHAR(255) NOT NULL
);

-- Application methods (amount/percentage off)
CREATE TABLE IF NOT EXISTS promotion_application_methods (
    id                  UUID         PRIMARY KEY DEFAULT uuid_generate_v4(),
    promotion_id        UUID         NOT NULL REFERENCES promotions(id) ON DELETE CASCADE,
    type                VARCHAR(50)  NOT NULL DEFAULT 'fixed',
    target_type         VARCHAR(50)  NOT NULL DEFAULT 'order',
    allocation          VARCHAR(50)  NOT NULL DEFAULT 'each',
    value               BIGINT,
    currency_code       VARCHAR(3),
    max_quantity        INT,
    apply_to_quantity   INT,
    buy_rules_min_quantity INT,
    created_at          TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);
