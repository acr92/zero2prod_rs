-- Create subscription tokens table
CREATE TABLE subscription_tokens
(
    token TEXT NOT NULL,
    id    UUID NOT NULL REFERENCES subscriptions (id),
    PRIMARY KEY (token)
);
