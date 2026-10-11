-- 使用会话心跳超时秒数，缺省 60
INSERT OR IGNORE INTO system_configs (id, config_key, config_value, scope, description, updated_at)
VALUES (
    'cfg-usage-session-timeout',
    'usage_session_timeout_seconds',
    '60',
    'global',
    '未结束使用会话的心跳超时秒数',
    datetime('now')
);
