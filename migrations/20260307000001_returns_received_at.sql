-- Add received_at column to returns table for tracking when returns are physically received
ALTER TABLE returns ADD COLUMN IF NOT EXISTS received_at TIMESTAMPTZ;
