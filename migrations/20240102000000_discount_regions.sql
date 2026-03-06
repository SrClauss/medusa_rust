-- Add discount_regions join table (discount ↔ region many-to-many)
CREATE TABLE IF NOT EXISTS discount_regions (
    discount_id UUID NOT NULL REFERENCES discounts(id),
    region_id   UUID NOT NULL REFERENCES regions(id),
    PRIMARY KEY (discount_id, region_id)
);
