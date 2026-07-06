CREATE TABLE IF NOT EXISTS repair_orders (
    id TEXT PRIMARY KEY,
    device_id TEXT NOT NULL DEFAULT '',
    device_name TEXT NOT NULL DEFAULT '',
    reporter TEXT NOT NULL DEFAULT '',
    issue_type TEXT NOT NULL DEFAULT 'hardware',
    description TEXT NOT NULL DEFAULT '',
    status TEXT NOT NULL DEFAULT 'pending',
    assigned_to TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    resolved_at TEXT,
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
