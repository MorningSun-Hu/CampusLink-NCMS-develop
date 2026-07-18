CREATE TABLE IF NOT EXISTS network_accounts (
    id TEXT PRIMARY KEY,
    account_name TEXT NOT NULL,
    password TEXT NOT NULL,
    device_id TEXT,
    status TEXT NOT NULL DEFAULT 'active',
    login_url TEXT,
    logout_url TEXT,
    auto_login INTEGER NOT NULL DEFAULT 0,
    auto_logout INTEGER NOT NULL DEFAULT 0,
    description TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
