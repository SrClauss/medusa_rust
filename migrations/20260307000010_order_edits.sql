-- Order Edits
CREATE TABLE IF NOT EXISTS order_edits (
    id                  UUID         PRIMARY KEY DEFAULT uuid_generate_v4(),
    order_id            UUID         NOT NULL REFERENCES orders(id) ON DELETE CASCADE,
    created_by          UUID,
    requested_by        UUID,
    confirmed_by        UUID,
    declined_by         UUID,
    canceled_by         UUID,
    status              VARCHAR(50)  NOT NULL DEFAULT 'created',
    internal_note       TEXT,
    declined_reason     TEXT,
    requested_at        TIMESTAMPTZ,
    confirmed_at        TIMESTAMPTZ,
    declined_at         TIMESTAMPTZ,
    canceled_at         TIMESTAMPTZ,
    difference_due      BIGINT,
    shipping_total      BIGINT       NOT NULL DEFAULT 0,
    discount_total      BIGINT       NOT NULL DEFAULT 0,
    tax_total           BIGINT       NOT NULL DEFAULT 0,
    subtotal            BIGINT       NOT NULL DEFAULT 0,
    total               BIGINT       NOT NULL DEFAULT 0,
    metadata            JSONB,
    created_at          TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

-- Order edit item changes
CREATE TABLE IF NOT EXISTS order_item_changes (
    id             UUID        PRIMARY KEY DEFAULT uuid_generate_v4(),
    type           VARCHAR(50) NOT NULL,
    order_edit_id  UUID        NOT NULL REFERENCES order_edits(id) ON DELETE CASCADE,
    original_line_item_id UUID,
    line_item_id   UUID,
    metadata       JSONB,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at     TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
