-- Add confirmed status to subscriptions
ALTER TABLE subscriptions
    ADD COLUMN confirmed BOOLEAN NOT NULL DEFAULT FALSE;
