CREATE TABLE IF NOT EXISTS kv_store
(
    key        VARCHAR(50) PRIMARY KEY,
    value      VARCHAR(1000) NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);