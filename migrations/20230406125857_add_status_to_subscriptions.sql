-- Add confirmed status to subscriptions
ALTER TABLE subscriptions
    ADD COLUMN confirmed BOOLEAN DEFAULT FALSE;
