-- MedusaRust Initial Schema
-- Mirrors the MedusaJS entity model so the same storefront and admin panel work.

-- ─── Extensions ───────────────────────────────────────────────────────────────
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- ─── Currencies ───────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS currencies (
    code            VARCHAR(3)   PRIMARY KEY,
    symbol          VARCHAR(10)  NOT NULL,
    symbol_native   VARCHAR(10)  NOT NULL,
    name            VARCHAR(100) NOT NULL,
    includes_tax    BOOLEAN      NOT NULL DEFAULT FALSE
);

INSERT INTO currencies (code, symbol, symbol_native, name) VALUES
    ('usd', '$',  '$',  'US Dollar'),
    ('eur', '€',  '€',  'Euro'),
    ('brl', 'R$', 'R$', 'Brazilian Real'),
    ('gbp', '£',  '£',  'British Pound')
ON CONFLICT DO NOTHING;

-- ─── Regions ──────────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS regions (
    id                UUID         PRIMARY KEY DEFAULT uuid_generate_v4(),
    name              VARCHAR(255) NOT NULL,
    currency_code     VARCHAR(3)   NOT NULL REFERENCES currencies(code),
    tax_rate          FLOAT        NOT NULL DEFAULT 0,
    tax_code          VARCHAR(255),
    gift_cards_taxable BOOLEAN     NOT NULL DEFAULT TRUE,
    automatic_taxes   BOOLEAN      NOT NULL DEFAULT TRUE,
    metadata          JSONB,
    created_at        TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at        TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    deleted_at        TIMESTAMPTZ
);

-- ─── Countries ────────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS countries (
    id           UUID         PRIMARY KEY DEFAULT uuid_generate_v4(),
    iso_2        VARCHAR(2)   NOT NULL UNIQUE,
    iso_3        VARCHAR(3)   NOT NULL,
    num_code     INT,
    name         VARCHAR(255) NOT NULL,
    display_name VARCHAR(255) NOT NULL,
    region_id    UUID         REFERENCES regions(id)
);

INSERT INTO countries (id, iso_2, iso_3, num_code, name, display_name) VALUES
    (uuid_generate_v4(), 'us', 'usa', 840, 'UNITED STATES', 'United States'),
    (uuid_generate_v4(), 'br', 'bra', 076, 'BRAZIL',        'Brazil'),
    (uuid_generate_v4(), 'de', 'deu', 276, 'GERMANY',       'Germany'),
    (uuid_generate_v4(), 'gb', 'gbr', 826, 'UNITED KINGDOM','United Kingdom'),
    (uuid_generate_v4(), 'fr', 'fra', 250, 'FRANCE',        'France')
ON CONFLICT DO NOTHING;

-- ─── Payment / Fulfillment providers ──────────────────────────────────────────
CREATE TABLE IF NOT EXISTS payment_providers (
    id           VARCHAR(255) PRIMARY KEY,
    is_installed BOOLEAN      NOT NULL DEFAULT TRUE
);
INSERT INTO payment_providers (id) VALUES ('manual') ON CONFLICT DO NOTHING;

CREATE TABLE IF NOT EXISTS fulfillment_providers (
    id           VARCHAR(255) PRIMARY KEY,
    is_installed BOOLEAN      NOT NULL DEFAULT TRUE
);
INSERT INTO fulfillment_providers (id) VALUES ('manual') ON CONFLICT DO NOTHING;

-- Region ↔ provider join tables
CREATE TABLE IF NOT EXISTS region_payment_providers (
    region_id   UUID         NOT NULL REFERENCES regions(id),
    provider_id VARCHAR(255) NOT NULL REFERENCES payment_providers(id),
    PRIMARY KEY (region_id, provider_id)
);

CREATE TABLE IF NOT EXISTS region_fulfillment_providers (
    region_id   UUID         NOT NULL REFERENCES regions(id),
    provider_id VARCHAR(255) NOT NULL REFERENCES fulfillment_providers(id),
    PRIMARY KEY (region_id, provider_id)
);

-- ─── Admin Users ──────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS users (
    id            UUID         PRIMARY KEY DEFAULT uuid_generate_v4(),
    email         VARCHAR(255) NOT NULL UNIQUE,
    first_name    VARCHAR(255),
    last_name     VARCHAR(255),
    role          VARCHAR(50)  NOT NULL DEFAULT 'member',
    password_hash VARCHAR(255),
    api_token     VARCHAR(255),
    metadata      JSONB,
    created_at    TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    deleted_at    TIMESTAMPTZ
);

-- ─── Customers ────────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS customers (
    id                 UUID         PRIMARY KEY DEFAULT uuid_generate_v4(),
    email              VARCHAR(255) NOT NULL,
    first_name         VARCHAR(255),
    last_name          VARCHAR(255),
    billing_address_id UUID,
    phone              VARCHAR(50),
    has_account        BOOLEAN      NOT NULL DEFAULT FALSE,
    password_hash      VARCHAR(255),
    metadata           JSONB,
    created_at         TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at         TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    deleted_at         TIMESTAMPTZ,
    UNIQUE (email, deleted_at)
);

-- ─── Addresses ────────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS addresses (
    id           UUID         PRIMARY KEY DEFAULT uuid_generate_v4(),
    customer_id  UUID         REFERENCES customers(id),
    first_name   VARCHAR(255),
    last_name    VARCHAR(255),
    phone        VARCHAR(50),
    company      VARCHAR(255),
    address_1    VARCHAR(255),
    address_2    VARCHAR(255),
    city         VARCHAR(255),
    country_code VARCHAR(2),
    province     VARCHAR(255),
    postal_code  VARCHAR(50),
    metadata     JSONB,
    created_at   TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at   TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

ALTER TABLE customers
    ADD CONSTRAINT fk_customers_billing_address
    FOREIGN KEY (billing_address_id) REFERENCES addresses(id)
    DEFERRABLE INITIALLY DEFERRED;

-- ─── Shipping Profiles ────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS shipping_profiles (
    id         UUID         PRIMARY KEY DEFAULT uuid_generate_v4(),
    name       VARCHAR(255) NOT NULL,
    type       VARCHAR(50)  NOT NULL DEFAULT 'default',
    metadata   JSONB,
    created_at TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

INSERT INTO shipping_profiles (id, name, type) VALUES
    (uuid_generate_v4(), 'Default Shipping Profile', 'default')
ON CONFLICT DO NOTHING;

-- ─── Shipping Options ─────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS shipping_options (
    id          UUID         PRIMARY KEY DEFAULT uuid_generate_v4(),
    name        VARCHAR(255) NOT NULL,
    region_id   UUID         NOT NULL REFERENCES regions(id),
    profile_id  UUID         NOT NULL REFERENCES shipping_profiles(id),
    provider_id VARCHAR(255) NOT NULL REFERENCES fulfillment_providers(id),
    price_type  VARCHAR(50)  NOT NULL DEFAULT 'flat_rate',
    amount      BIGINT,
    is_return   BOOLEAN      NOT NULL DEFAULT FALSE,
    admin_only  BOOLEAN      NOT NULL DEFAULT FALSE,
    data        JSONB        NOT NULL DEFAULT '{}',
    metadata    JSONB,
    created_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    deleted_at  TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS shipping_option_requirements (
    id                 UUID        PRIMARY KEY DEFAULT uuid_generate_v4(),
    shipping_option_id UUID        NOT NULL REFERENCES shipping_options(id),
    type               VARCHAR(50) NOT NULL,
    amount             BIGINT      NOT NULL
);

-- ─── Product Types ────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS product_types (
    id         UUID         PRIMARY KEY DEFAULT uuid_generate_v4(),
    value      VARCHAR(255) NOT NULL UNIQUE,
    metadata   JSONB,
    created_at TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

-- ─── Product Tags ─────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS product_tags (
    id         UUID         PRIMARY KEY DEFAULT uuid_generate_v4(),
    value      VARCHAR(255) NOT NULL,
    metadata   JSONB,
    created_at TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

-- ─── Product Images ───────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS product_images (
    id         UUID         PRIMARY KEY DEFAULT uuid_generate_v4(),
    url        TEXT         NOT NULL,
    variant_id UUID,
    filename   VARCHAR(255),
    size_bytes BIGINT,
    metadata   JSONB,
    created_at TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

-- ─── Product Collections ──────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS product_collections (
    id         UUID         PRIMARY KEY DEFAULT uuid_generate_v4(),
    title      VARCHAR(255) NOT NULL,
    handle     VARCHAR(255) NOT NULL UNIQUE,
    metadata   JSONB,
    created_at TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

-- ─── Product Categories ───────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS product_categories (
    id                 UUID         PRIMARY KEY DEFAULT uuid_generate_v4(),
    name               VARCHAR(255) NOT NULL,
    description        TEXT,
    handle             VARCHAR(255) NOT NULL UNIQUE,
    is_active          BOOLEAN      NOT NULL DEFAULT TRUE,
    is_internal        BOOLEAN      NOT NULL DEFAULT FALSE,
    parent_category_id UUID         REFERENCES product_categories(id),
    rank               INT          NOT NULL DEFAULT 0,
    metadata           JSONB,
    created_at         TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at         TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

-- ─── Products ─────────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS products (
    id             UUID         PRIMARY KEY DEFAULT uuid_generate_v4(),
    title          VARCHAR(255) NOT NULL,
    subtitle       VARCHAR(255),
    description    TEXT,
    handle         VARCHAR(255) NOT NULL UNIQUE,
    is_giftcard    BOOLEAN      NOT NULL DEFAULT FALSE,
    status         VARCHAR(50)  NOT NULL DEFAULT 'draft',
    thumbnail      TEXT,
    weight         FLOAT,
    length         FLOAT,
    height         FLOAT,
    width          FLOAT,
    hs_code        VARCHAR(255),
    origin_country VARCHAR(255),
    mid_code       VARCHAR(255),
    material       VARCHAR(255),
    collection_id  UUID         REFERENCES product_collections(id),
    type_id        UUID         REFERENCES product_types(id),
    discountable   BOOLEAN      NOT NULL DEFAULT TRUE,
    external_id    VARCHAR(255),
    metadata       JSONB,
    created_at     TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at     TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    deleted_at     TIMESTAMPTZ
);

-- Product ↔ Image join table
CREATE TABLE IF NOT EXISTS product_images_products (
    product_id UUID NOT NULL REFERENCES products(id),
    image_id   UUID NOT NULL REFERENCES product_images(id),
    PRIMARY KEY (product_id, image_id)
);

-- Product ↔ Tag join table
CREATE TABLE IF NOT EXISTS product_tags_products (
    product_id UUID NOT NULL REFERENCES products(id),
    tag_id     UUID NOT NULL REFERENCES product_tags(id),
    PRIMARY KEY (product_id, tag_id)
);

-- Product ↔ Category join table
CREATE TABLE IF NOT EXISTS product_category_products (
    product_id  UUID NOT NULL REFERENCES products(id),
    category_id UUID NOT NULL REFERENCES product_categories(id),
    PRIMARY KEY (product_id, category_id)
);

-- ─── Product Options ──────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS product_options (
    id         UUID         PRIMARY KEY DEFAULT uuid_generate_v4(),
    title      VARCHAR(255) NOT NULL,
    product_id UUID         NOT NULL REFERENCES products(id),
    metadata   JSONB,
    created_at TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

-- ─── Product Variants ─────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS product_variants (
    id                 UUID         PRIMARY KEY DEFAULT uuid_generate_v4(),
    title              VARCHAR(255) NOT NULL,
    product_id         UUID         NOT NULL REFERENCES products(id),
    sku                VARCHAR(255) UNIQUE,
    barcode            VARCHAR(255),
    ean                VARCHAR(255),
    upc                VARCHAR(255),
    inventory_quantity INT          NOT NULL DEFAULT 0,
    allow_backorder    BOOLEAN      NOT NULL DEFAULT FALSE,
    manage_inventory   BOOLEAN      NOT NULL DEFAULT TRUE,
    hs_code            VARCHAR(255),
    origin_country     VARCHAR(255),
    mid_code           VARCHAR(255),
    material           VARCHAR(255),
    weight             FLOAT,
    length             FLOAT,
    height             FLOAT,
    width              FLOAT,
    variant_rank       INT,
    metadata           JSONB,
    created_at         TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at         TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    deleted_at         TIMESTAMPTZ
);

-- ─── Product Option Values ────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS product_option_values (
    id         UUID         PRIMARY KEY DEFAULT uuid_generate_v4(),
    value      VARCHAR(255) NOT NULL,
    option_id  UUID         NOT NULL REFERENCES product_options(id),
    variant_id UUID         NOT NULL REFERENCES product_variants(id),
    metadata   JSONB,
    created_at TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

-- ─── Money Amounts (Prices) ───────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS money_amounts (
    id            UUID        PRIMARY KEY DEFAULT uuid_generate_v4(),
    currency_code VARCHAR(3)  NOT NULL REFERENCES currencies(code),
    amount        BIGINT      NOT NULL,
    variant_id    UUID        NOT NULL REFERENCES product_variants(id),
    region_id     UUID        REFERENCES regions(id),
    price_list_id UUID,
    min_quantity  INT,
    max_quantity  INT,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at    TIMESTAMPTZ
);

-- ─── Price Lists ──────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS price_lists (
    id              UUID         PRIMARY KEY DEFAULT uuid_generate_v4(),
    name            VARCHAR(255) NOT NULL,
    description     TEXT         NOT NULL DEFAULT '',
    price_list_type VARCHAR(50)  NOT NULL DEFAULT 'sale',
    status          VARCHAR(50)  NOT NULL DEFAULT 'active',
    starts_at       TIMESTAMPTZ,
    ends_at         TIMESTAMPTZ,
    created_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    deleted_at      TIMESTAMPTZ
);

ALTER TABLE money_amounts
    ADD CONSTRAINT fk_money_amounts_price_list
    FOREIGN KEY (price_list_id) REFERENCES price_lists(id)
    DEFERRABLE INITIALLY DEFERRED;

-- ─── Carts ────────────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS carts (
    id                     UUID         PRIMARY KEY DEFAULT uuid_generate_v4(),
    email                  VARCHAR(255),
    billing_address_id     UUID         REFERENCES addresses(id),
    shipping_address_id    UUID         REFERENCES addresses(id),
    region_id              UUID         NOT NULL REFERENCES regions(id),
    customer_id            UUID         REFERENCES customers(id),
    payment_id             UUID,
    cart_type              VARCHAR(50)  NOT NULL DEFAULT 'default',
    completed_at           TIMESTAMPTZ,
    payment_authorized_at  TIMESTAMPTZ,
    idempotency_key        VARCHAR(255),
    context                JSONB,
    sales_channel_id       UUID,
    metadata               JSONB,
    created_at             TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at             TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    deleted_at             TIMESTAMPTZ
);

-- ─── Discounts ────────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS discount_rules (
    id          UUID         PRIMARY KEY DEFAULT uuid_generate_v4(),
    description VARCHAR(255),
    type        VARCHAR(50)  NOT NULL,
    value       BIGINT       NOT NULL DEFAULT 0,
    allocation  VARCHAR(50),
    metadata    JSONB,
    created_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    deleted_at  TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS discounts (
    id                 UUID         PRIMARY KEY DEFAULT uuid_generate_v4(),
    code               VARCHAR(255) NOT NULL UNIQUE,
    is_dynamic         BOOLEAN      NOT NULL DEFAULT FALSE,
    rule_id            UUID         REFERENCES discount_rules(id),
    is_disabled        BOOLEAN      NOT NULL DEFAULT FALSE,
    parent_discount_id UUID         REFERENCES discounts(id),
    starts_at          TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    ends_at            TIMESTAMPTZ,
    valid_duration     VARCHAR(255),
    usage_limit        INT,
    usage_count        INT          NOT NULL DEFAULT 0,
    metadata           JSONB,
    created_at         TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at         TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    deleted_at         TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS discount_conditions (
    id               UUID         PRIMARY KEY DEFAULT uuid_generate_v4(),
    discount_rule_id UUID         NOT NULL REFERENCES discount_rules(id),
    type             VARCHAR(50)  NOT NULL,
    operator         VARCHAR(50)  NOT NULL,
    metadata         JSONB,
    created_at       TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at       TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    deleted_at       TIMESTAMPTZ
);

-- Cart ↔ Discount join table
CREATE TABLE IF NOT EXISTS cart_discounts (
    cart_id     UUID NOT NULL REFERENCES carts(id),
    discount_id UUID NOT NULL REFERENCES discounts(id),
    PRIMARY KEY (cart_id, discount_id)
);

-- ─── Gift Cards ───────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS gift_cards (
    id          UUID         PRIMARY KEY DEFAULT uuid_generate_v4(),
    code        VARCHAR(255) NOT NULL UNIQUE,
    value       BIGINT       NOT NULL,
    balance     BIGINT       NOT NULL,
    region_id   UUID         NOT NULL REFERENCES regions(id),
    order_id    UUID,
    is_disabled BOOLEAN      NOT NULL DEFAULT FALSE,
    ends_at     TIMESTAMPTZ,
    metadata    JSONB,
    created_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    deleted_at  TIMESTAMPTZ
);

-- ─── Line Items ───────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS line_items (
    id                 UUID         PRIMARY KEY DEFAULT uuid_generate_v4(),
    cart_id            UUID         REFERENCES carts(id),
    order_id           UUID,
    swap_id            UUID,
    claim_order_id     UUID,
    title              VARCHAR(255) NOT NULL,
    description        VARCHAR(255),
    thumbnail          TEXT,
    is_return          BOOLEAN      NOT NULL DEFAULT FALSE,
    is_giftcard        BOOLEAN      NOT NULL DEFAULT FALSE,
    should_merge       BOOLEAN      NOT NULL DEFAULT TRUE,
    allow_discounts    BOOLEAN      NOT NULL DEFAULT TRUE,
    has_shipping       BOOLEAN,
    unit_price         BIGINT       NOT NULL,
    variant_id         UUID         REFERENCES product_variants(id),
    quantity           INT          NOT NULL DEFAULT 1,
    fulfilled_quantity INT,
    returned_quantity  INT,
    shipped_quantity   INT,
    metadata           JSONB,
    original_item_id   UUID,
    order_edit_id      UUID,
    created_at         TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at         TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS line_item_adjustments (
    id          UUID  PRIMARY KEY DEFAULT uuid_generate_v4(),
    item_id     UUID  NOT NULL REFERENCES line_items(id),
    description TEXT  NOT NULL,
    discount_id UUID  REFERENCES discounts(id),
    amount      BIGINT NOT NULL,
    metadata    JSONB
);

CREATE TABLE IF NOT EXISTS line_item_tax_lines (
    id         UUID         PRIMARY KEY DEFAULT uuid_generate_v4(),
    rate       FLOAT        NOT NULL,
    name       VARCHAR(255) NOT NULL,
    code       VARCHAR(255),
    item_id    UUID         NOT NULL REFERENCES line_items(id),
    created_at TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

-- ─── Payment Sessions ─────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS payment_sessions (
    id                     UUID         PRIMARY KEY DEFAULT uuid_generate_v4(),
    cart_id                UUID         REFERENCES carts(id),
    provider_id            VARCHAR(255) NOT NULL,
    is_selected            BOOLEAN      DEFAULT FALSE,
    is_initiated           BOOLEAN      NOT NULL DEFAULT FALSE,
    status                 VARCHAR(50)  NOT NULL DEFAULT 'pending',
    data                   JSONB        NOT NULL DEFAULT '{}',
    idempotency_key        VARCHAR(255),
    amount                 BIGINT,
    payment_authorized_at  TIMESTAMPTZ,
    created_at             TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at             TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    UNIQUE (cart_id, provider_id)
);

-- ─── Payments ─────────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS payments (
    id              UUID         PRIMARY KEY DEFAULT uuid_generate_v4(),
    swap_id         UUID,
    cart_id         UUID         REFERENCES carts(id),
    order_id        UUID,
    amount          BIGINT       NOT NULL,
    currency_code   VARCHAR(3)   NOT NULL,
    amount_refunded BIGINT       NOT NULL DEFAULT 0,
    provider_id     VARCHAR(255) NOT NULL,
    data            JSONB        NOT NULL DEFAULT '{}',
    captured_at     TIMESTAMPTZ,
    cancelled_at    TIMESTAMPTZ,
    metadata        JSONB,
    idempotency_key VARCHAR(255),
    created_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

-- ─── Orders ───────────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS orders (
    id                 UUID         PRIMARY KEY DEFAULT uuid_generate_v4(),
    status             VARCHAR(50)  NOT NULL DEFAULT 'pending',
    fulfillment_status VARCHAR(50)  NOT NULL DEFAULT 'not_fulfilled',
    payment_status     VARCHAR(50)  NOT NULL DEFAULT 'not_paid',
    display_id         INT          NOT NULL,
    cart_id            UUID         REFERENCES carts(id),
    customer_id        UUID         NOT NULL,
    email              VARCHAR(255) NOT NULL,
    billing_address_id UUID         REFERENCES addresses(id),
    shipping_address_id UUID        REFERENCES addresses(id),
    region_id          UUID         NOT NULL REFERENCES regions(id),
    currency_code      VARCHAR(3)   NOT NULL,
    tax_rate           FLOAT,
    canceled_at        TIMESTAMPTZ,
    metadata           JSONB,
    no_notification    BOOLEAN,
    idempotency_key    VARCHAR(255),
    external_id        VARCHAR(255),
    sales_channel_id   UUID,
    created_at         TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at         TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

ALTER TABLE line_items ADD CONSTRAINT fk_line_items_order FOREIGN KEY (order_id) REFERENCES orders(id) DEFERRABLE INITIALLY DEFERRED;
ALTER TABLE payments  ADD CONSTRAINT fk_payments_order   FOREIGN KEY (order_id) REFERENCES orders(id) DEFERRABLE INITIALLY DEFERRED;
ALTER TABLE gift_cards ADD CONSTRAINT fk_gift_cards_order FOREIGN KEY (order_id) REFERENCES orders(id) DEFERRABLE INITIALLY DEFERRED;

-- ─── Fulfillments ─────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS fulfillments (
    id               UUID         PRIMARY KEY DEFAULT uuid_generate_v4(),
    claim_order_id   UUID,
    swap_id          UUID,
    order_id         UUID         REFERENCES orders(id),
    provider_id      VARCHAR(255) NOT NULL DEFAULT 'manual',
    location_id      UUID,
    no_notification  BOOLEAN,
    tracking_numbers JSONB,
    data             JSONB,
    shipped_at       TIMESTAMPTZ,
    canceled_at      TIMESTAMPTZ,
    metadata         JSONB,
    idempotency_key  VARCHAR(255),
    created_at       TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at       TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

-- ─── Returns ──────────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS returns (
    id                UUID         PRIMARY KEY DEFAULT uuid_generate_v4(),
    status            VARCHAR(50)  NOT NULL DEFAULT 'requested',
    order_id          UUID         REFERENCES orders(id),
    swap_id           UUID,
    claim_order_id    UUID,
    shipping_method_id UUID,
    refund_amount     BIGINT,
    no_notification   BOOLEAN,
    idempotency_key   VARCHAR(255),
    metadata          JSONB,
    created_at        TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at        TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

-- ─── Swaps ────────────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS swaps (
    id                 UUID         PRIMARY KEY DEFAULT uuid_generate_v4(),
    fulfillment_status VARCHAR(50)  NOT NULL DEFAULT 'not_fulfilled',
    payment_status     VARCHAR(50)  NOT NULL DEFAULT 'not_paid',
    order_id           UUID         NOT NULL REFERENCES orders(id),
    difference_due     BIGINT,
    cart_id            UUID         REFERENCES carts(id),
    confirmed_at       TIMESTAMPTZ,
    canceled_at        TIMESTAMPTZ,
    no_notification    BOOLEAN,
    allow_backorder    BOOLEAN      NOT NULL DEFAULT FALSE,
    idempotency_key    VARCHAR(255),
    created_at         TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at         TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

-- ─── Claim Orders ─────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS claim_orders (
    id                  UUID         PRIMARY KEY DEFAULT uuid_generate_v4(),
    payment_status      VARCHAR(50)  NOT NULL DEFAULT 'na',
    fulfillment_status  VARCHAR(50)  NOT NULL DEFAULT 'not_fulfilled',
    type                VARCHAR(50)  NOT NULL,
    order_id            UUID         NOT NULL REFERENCES orders(id),
    shipping_address_id UUID         REFERENCES addresses(id),
    refund_amount       BIGINT,
    canceled_at         TIMESTAMPTZ,
    created_at          TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

-- ─── Refunds ──────────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS refunds (
    id              UUID         PRIMARY KEY DEFAULT uuid_generate_v4(),
    order_id        UUID         NOT NULL REFERENCES orders(id),
    amount          BIGINT       NOT NULL,
    note            TEXT,
    reason          VARCHAR(50)  NOT NULL DEFAULT 'other',
    payment_id      UUID,
    metadata        JSONB,
    idempotency_key VARCHAR(255),
    created_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

-- ─── Shipping Methods ─────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS shipping_methods (
    id                 UUID   PRIMARY KEY DEFAULT uuid_generate_v4(),
    shipping_option_id UUID   NOT NULL REFERENCES shipping_options(id),
    cart_id            UUID   REFERENCES carts(id),
    order_id           UUID   REFERENCES orders(id),
    claim_order_id     UUID,
    swap_id            UUID,
    return_id          UUID,
    price              BIGINT NOT NULL,
    data               JSONB  NOT NULL DEFAULT '{}'
);

-- ─── Tax Rates ────────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS tax_rates (
    id         UUID         PRIMARY KEY DEFAULT uuid_generate_v4(),
    rate       FLOAT,
    code       VARCHAR(255),
    name       VARCHAR(255) NOT NULL,
    region_id  UUID         NOT NULL REFERENCES regions(id),
    metadata   JSONB,
    created_at TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

-- ─── Inventory Items ──────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS inventory_items (
    id               UUID         PRIMARY KEY DEFAULT uuid_generate_v4(),
    sku              VARCHAR(255),
    origin_country   VARCHAR(255),
    hs_code          VARCHAR(255),
    mid_code         VARCHAR(255),
    material         VARCHAR(255),
    weight           FLOAT,
    length           FLOAT,
    height           FLOAT,
    width            FLOAT,
    requires_shipping BOOLEAN     NOT NULL DEFAULT TRUE,
    description      TEXT,
    thumbnail        TEXT,
    title            VARCHAR(255),
    metadata         JSONB,
    created_at       TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at       TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    deleted_at       TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS inventory_levels (
    id                  UUID        PRIMARY KEY DEFAULT uuid_generate_v4(),
    inventory_item_id   UUID        NOT NULL REFERENCES inventory_items(id),
    location_id         UUID        NOT NULL,
    stocked_quantity    INT         NOT NULL DEFAULT 0,
    reserved_quantity   INT         NOT NULL DEFAULT 0,
    incoming_quantity   INT         NOT NULL DEFAULT 0,
    metadata            JSONB,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (inventory_item_id, location_id)
);

-- ─── Draft Orders ─────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS draft_orders (
    id                    UUID         PRIMARY KEY DEFAULT uuid_generate_v4(),
    status                VARCHAR(50)  NOT NULL DEFAULT 'open',
    display_id            INT          NOT NULL DEFAULT 1,
    cart_id               UUID         REFERENCES carts(id),
    order_id              UUID         REFERENCES orders(id),
    canceled_at           TIMESTAMPTZ,
    completed_at          TIMESTAMPTZ,
    no_notification_order BOOLEAN,
    idempotency_key       VARCHAR(255),
    metadata              JSONB,
    created_at            TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at            TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

-- ─── Batch Jobs ───────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS batch_jobs (
    id               UUID         PRIMARY KEY DEFAULT uuid_generate_v4(),
    type             VARCHAR(255) NOT NULL,
    created_by       UUID,
    context          JSONB,
    result           JSONB,
    dry_run          BOOLEAN      NOT NULL DEFAULT FALSE,
    status           VARCHAR(50)  NOT NULL DEFAULT 'created',
    pre_processed_at TIMESTAMPTZ,
    processing_at    TIMESTAMPTZ,
    confirmed_at     TIMESTAMPTZ,
    completed_at     TIMESTAMPTZ,
    failed_at        TIMESTAMPTZ,
    canceled_at      TIMESTAMPTZ,
    created_at       TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at       TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

-- ─── Indexes ──────────────────────────────────────────────────────────────────
CREATE INDEX IF NOT EXISTS idx_products_handle       ON products (handle)        WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_products_status       ON products (status)        WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_products_collection   ON products (collection_id) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_variants_product      ON product_variants (product_id) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_variants_sku          ON product_variants (sku)   WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_money_amounts_variant ON money_amounts (variant_id)  WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_carts_customer        ON carts (customer_id)      WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_orders_customer       ON orders (customer_id);
CREATE INDEX IF NOT EXISTS idx_orders_cart           ON orders (cart_id);
CREATE INDEX IF NOT EXISTS idx_orders_display_id     ON orders (display_id);
CREATE INDEX IF NOT EXISTS idx_line_items_cart       ON line_items (cart_id);
CREATE INDEX IF NOT EXISTS idx_line_items_order      ON line_items (order_id);
CREATE INDEX IF NOT EXISTS idx_customers_email       ON customers (email)        WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_shipping_options_region ON shipping_options (region_id) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_discounts_code        ON discounts (code)         WHERE deleted_at IS NULL;
