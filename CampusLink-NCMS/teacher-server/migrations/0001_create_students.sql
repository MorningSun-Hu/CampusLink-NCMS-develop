-- 创建 students 表
-- 用途：保存学生基础信息与登录凭据

CREATE TABLE students (
    id TEXT PRIMARY KEY,
    student_no TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    password_hash TEXT NOT NULL,
    seat_no TEXT,
    status TEXT NOT NULL DEFAULT 'active',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- 创建 status 索引
CREATE INDEX idx_students_status ON students(status);
