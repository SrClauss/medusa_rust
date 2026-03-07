-- Stores (multi-store support)
CREATE TABLE IF NOT EXISTS stores (
    id                     UUID         PRIMARY KEY DEFAULT uuid_generate_v4(),
    name                   VARCHAR(255) NOT NULL DEFAULT 'Medusa Store',
    default_currency_code  VARCHAR(3)   REFERENCES currencies(code),
    swap_link_template     VARCHAR(255),
    payment_link_template  VARCHAR(255),
    invite_link_template   VARCHAR(255),
    metadata               JSONB,
    created_at             TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at             TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

INSERT INTO stores (id, name, default_currency_code)
VALUES (uuid_generate_v4(), 'Medusa Store', 'usd')
ON CONFLICT DO NOTHING;
