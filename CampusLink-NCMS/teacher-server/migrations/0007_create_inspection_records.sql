-- 创建 inspection_records 表
-- 用途：保存卫生检查和设备检查记录

CREATE TABLE inspection_records (
    id TEXT PRIMARY KEY,
    device_id TEXT NOT NULL,
    student_id TEXT,
    inspection_type TEXT NOT NULL,
    item_name TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'normal',
    description TEXT,
    photo_url TEXT,
    inspector TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (device_id) REFERENCES student_devices(id) ON DELETE CASCADE,
    FOREIGN KEY (student_id) REFERENCES students(id) ON DELETE SET NULL
);

CREATE INDEX idx_inspection_device_id ON inspection_records(device_id);
CREATE INDEX idx_inspection_type ON inspection_records(inspection_type);
CREATE INDEX idx_inspection_status ON inspection_records(status);
CREATE INDEX idx_inspection_created_at ON inspection_records(created_at);
