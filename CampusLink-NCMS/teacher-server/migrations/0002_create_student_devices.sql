-- 创建 student_devices 表
-- 用途：保存学生机注册信息、在线状态和当前模式

CREATE TABLE student_devices (
    id TEXT PRIMARY KEY,
    student_id TEXT,
    device_code TEXT NOT NULL UNIQUE,
    device_name TEXT NOT NULL,
    machine_fingerprint TEXT NOT NULL UNIQUE,
    hostname TEXT NOT NULL,
    ip_address TEXT NOT NULL,
    mac_address TEXT NOT NULL,
    register_status TEXT NOT NULL DEFAULT 'pending',
    online_status TEXT NOT NULL DEFAULT 'offline',
    last_seen_at TEXT,
    current_mode TEXT NOT NULL DEFAULT 'open',
    agent_version TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (student_id) REFERENCES students(id) ON DELETE SET NULL
);

-- 创建 student_id 索引
CREATE INDEX idx_student_devices_student_id ON student_devices(student_id);

-- 创建 online_status 索引
CREATE INDEX idx_student_devices_online_status ON student_devices(online_status);

-- 创建 last_seen_at 索引
CREATE INDEX idx_student_devices_last_seen_at ON student_devices(last_seen_at);
