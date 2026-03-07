-- Price Preferences (per-region/currency price configuration)
CREATE TABLE IF NOT EXISTS price_preferences (
    id            UUID         PRIMARY KEY DEFAULT uuid_generate_v4(),
    attribute     VARCHAR(255) NOT NULL,
    value         VARCHAR(255) NOT NULL,
    is_tax_inclusive BOOLEAN   NOT NULL DEFAULT FALSE,
    created_at    TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

-- Exchanges (order exchanges)
CREATE TABLE IF NOT EXISTS exchanges (
    id                 UUID         PRIMARY KEY DEFAULT uuid_generate_v4(),
    order_id           UUID         NOT NULL REFERENCES orders(id) ON DELETE CASCADE,
    payment_status     VARCHAR(50)  NOT NULL DEFAULT 'not_paid',
    fulfillment_status VARCHAR(50)  NOT NULL DEFAULT 'not_fulfilled',
    difference_due     BIGINT,
    cart_id            UUID         REFERENCES carts(id),
    canceled_at        TIMESTAMPTZ,
    no_notification    BOOLEAN,
    allow_backorder    BOOLEAN      NOT NULL DEFAULT FALSE,
    metadata           JSONB,
    created_at         TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at         TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

-- Refund reasons
CREATE TABLE IF NOT EXISTS refund_reasons (
    id          UUID         PRIMARY KEY DEFAULT uuid_generate_v4(),
    label       VARCHAR(255) NOT NULL,
    description TEXT,
    metadata    JSONB,
    created_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    deleted_at  TIMESTAMPTZ
);

INSERT INTO refund_reasons (id, label, description) VALUES
    (uuid_generate_v4(), 'Duplicate',     'Item was ordered twice'),
    (uuid_generate_v4(), 'Fraudulent',    'Unauthorized transaction'),
    (uuid_generate_v4(), 'Customer Request', 'Customer changed mind'),
    (uuid_generate_v4(), 'Other',         'Other reason')
ON CONFLICT DO NOTHING;
