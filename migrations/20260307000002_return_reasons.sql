-- ─── Return Reasons ───────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS return_reasons (
    id          UUID         PRIMARY KEY DEFAULT uuid_generate_v4(),
    value       VARCHAR(255) NOT NULL UNIQUE,
    label       VARCHAR(255) NOT NULL,
    description TEXT,
    parent_return_reason_id UUID REFERENCES return_reasons(id) ON DELETE SET NULL,
    metadata    JSONB,
    created_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    deleted_at  TIMESTAMPTZ
);
