-- Notifications
CREATE TABLE IF NOT EXISTS notifications (
    id           UUID         PRIMARY KEY DEFAULT uuid_generate_v4(),
    to_address   VARCHAR(255) NOT NULL,
    channel      VARCHAR(50)  NOT NULL DEFAULT 'email',
    template     VARCHAR(255) NOT NULL,
    data         JSONB,
    trigger_type VARCHAR(255),
    resource_id  UUID,
    resource_type VARCHAR(255),
    receiver_id  UUID,
    original_notification_id UUID REFERENCES notifications(id),
    idempotency_key VARCHAR(255),
    external_id  VARCHAR(255),
    status       VARCHAR(50)  NOT NULL DEFAULT 'pending',
    provider_id  VARCHAR(255),
    created_at   TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at   TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_notifications_receiver ON notifications (receiver_id);
CREATE INDEX IF NOT EXISTS idx_notifications_resource ON notifications (resource_id, resource_type);
