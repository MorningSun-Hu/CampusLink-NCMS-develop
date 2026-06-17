-- 创建 process_guard_policies 表
-- 用途：存储进程守护策略

CREATE TABLE process_guard_policies (
    id TEXT PRIMARY KEY,
    device_id TEXT,
    process_name TEXT NOT NULL,
    check_interval_seconds INTEGER NOT NULL DEFAULT 60,
    max_restart_attempts INTEGER NOT NULL DEFAULT 3,
    enabled INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (device_id) REFERENCES student_devices(id) ON DELETE CASCADE
);

CREATE INDEX idx_process_guard_policies_device_id ON process_guard_policies(device_id);
CREATE INDEX idx_process_guard_policies_enabled ON process_guard_policies(enabled);
