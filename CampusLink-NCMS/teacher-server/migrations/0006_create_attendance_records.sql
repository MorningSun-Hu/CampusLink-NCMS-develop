-- 创建 attendance_records 表
-- 用途：保存课堂签到记录

CREATE TABLE attendance_records (
    id TEXT PRIMARY KEY,
    student_id TEXT,
    device_id TEXT NOT NULL,
    check_in_time TEXT NOT NULL DEFAULT (datetime('now')),
    check_out_time TEXT,
    status TEXT NOT NULL DEFAULT 'present',
    remarks TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (student_id) REFERENCES students(id) ON DELETE SET NULL,
    FOREIGN KEY (device_id) REFERENCES student_devices(id) ON DELETE CASCADE
);

CREATE INDEX idx_attendance_student_id ON attendance_records(student_id);
CREATE INDEX idx_attendance_device_id ON attendance_records(device_id);
CREATE INDEX idx_attendance_status ON attendance_records(status);
CREATE INDEX idx_attendance_check_in_time ON attendance_records(check_in_time);
