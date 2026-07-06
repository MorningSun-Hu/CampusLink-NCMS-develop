CREATE TABLE IF NOT EXISTS device_whitelist (
    id TEXT PRIMARY KEY,
    device_code TEXT NOT NULL UNIQUE,
    device_name TEXT NOT NULL DEFAULT '',
    mac_address TEXT NOT NULL DEFAULT '',
    status TEXT NOT NULL DEFAULT 'pending',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
