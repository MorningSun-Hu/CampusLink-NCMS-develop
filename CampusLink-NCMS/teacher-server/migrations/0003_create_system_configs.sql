-- 创建 system_configs 表
-- 用途：保存教师端全局配置与模式基础参数

CREATE TABLE system_configs (
    id TEXT PRIMARY KEY,
    config_key TEXT NOT NULL UNIQUE,
    config_value TEXT NOT NULL,
    scope TEXT NOT NULL DEFAULT 'global',
    description TEXT,
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- 创建 scope 索引
CREATE INDEX idx_system_configs_scope ON system_configs(scope);
