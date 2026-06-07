-- 创建 operation_logs 表
-- 用途：记录注册、模式切换、系统操作等行为

CREATE TABLE operation_logs (
    id TEXT PRIMARY KEY,
    log_type TEXT NOT NULL,
    operator TEXT NOT NULL,
    target_id TEXT,
    content TEXT NOT NULL,
    extra_payload TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- 创建 log_type 索引
CREATE INDEX idx_operation_logs_log_type ON operation_logs(log_type);

-- 创建 target_id 索引
CREATE INDEX idx_operation_logs_target_id ON operation_logs(target_id);

-- 创建 created_at 索引
CREATE INDEX idx_operation_logs_created_at ON operation_logs(created_at);
