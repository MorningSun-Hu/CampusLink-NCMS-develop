-- 创建 hardware_snapshots 表
-- 用途：保存学生端硬件快照信息

CREATE TABLE hardware_snapshots (
    id TEXT PRIMARY KEY,
    device_id TEXT NOT NULL,
    cpu_model TEXT,
    cpu_cores INTEGER,
    total_memory_bytes INTEGER,
    disk_info TEXT,
    mac_addresses TEXT,
    gpu_info TEXT,
    os_version TEXT,
    hostname TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (device_id) REFERENCES student_devices(id) ON DELETE CASCADE
);

CREATE INDEX idx_hardware_snapshots_device_id ON hardware_snapshots(device_id);
CREATE INDEX idx_hardware_snapshots_created_at ON hardware_snapshots(created_at);
