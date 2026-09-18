-- 设备使用记录与考勤记录关联字段
-- 用途：记录学生机从签到开始的使用会话，供教师端查询、筛选与导出

CREATE TABLE IF NOT EXISTS device_usage_records (
    id TEXT PRIMARY KEY,
    device_id TEXT NOT NULL REFERENCES student_devices(id) ON DELETE CASCADE,
    class_id TEXT REFERENCES classes(id) ON DELETE SET NULL,
    seat_no TEXT,
    student_id TEXT REFERENCES students(id) ON DELETE SET NULL,
    user_name TEXT NOT NULL,
    mode TEXT NOT NULL,
    start_time TEXT NOT NULL,
    end_time TEXT,
    duration_seconds INTEGER,
    inspection_ok INTEGER NOT NULL DEFAULT 1,
    inspection_summary TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_usage_device_time ON device_usage_records(device_id, start_time DESC);
CREATE INDEX IF NOT EXISTS idx_usage_start_time ON device_usage_records(start_time);
CREATE INDEX IF NOT EXISTS idx_usage_class_id ON device_usage_records(class_id);

-- 同一设备同一时刻至多存在一条未结束的使用会话
CREATE UNIQUE INDEX IF NOT EXISTS idx_usage_open_session ON device_usage_records(device_id) WHERE end_time IS NULL;

-- 考勤记录关联对应的使用会话
ALTER TABLE attendance_records ADD COLUMN usage_record_id TEXT;
