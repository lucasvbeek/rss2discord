-- Add migration script here
CREATE TABLE IF NOT EXISTS feed_items
(
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    feed_name VARCHAR (32) NOT NULL,
    external_id VARCHAR NOT NULL,
    published_at TIMESTAMP WITH TIME ZONE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    variables JSONB,
    UNIQUE (feed_name, external_id)
)