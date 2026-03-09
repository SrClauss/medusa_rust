-- Multi-language: languages and product_translations tables

CREATE TABLE IF NOT EXISTS languages (
    code       TEXT        PRIMARY KEY,  -- e.g. "en", "pt-BR"
    name       TEXT        NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS product_translations (
    id            UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    product_id    UUID        NOT NULL REFERENCES products(id) ON DELETE CASCADE,
    language_code TEXT        NOT NULL REFERENCES languages(code) ON DELETE CASCADE,
    title         TEXT,
    description   TEXT,
    UNIQUE (product_id, language_code)
);

CREATE INDEX IF NOT EXISTS idx_product_translations_product ON product_translations(product_id);
