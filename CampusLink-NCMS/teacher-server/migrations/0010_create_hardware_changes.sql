-- 创建 hardware_changes 表
-- 用途：记录设备硬件变更历史

CREATE TABLE hardware_changes (
    id TEXT PRIMARY KEY,
    device_id TEXT NOT NULL,
    change_type TEXT NOT NULL,
    field_name TEXT NOT NULL,
    old_value TEXT,
    new_value TEXT,
    detected_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (device_id) REFERENCES student_devices(id) ON DELETE CASCADE
);

CREATE INDEX idx_hardware_changes_device_id ON hardware_changes(device_id);
CREATE INDEX idx_hardware_changes_change_type ON hardware_changes(change_type);
CREATE INDEX idx_hardware_changes_detected_at ON hardware_changes(detected_at);
