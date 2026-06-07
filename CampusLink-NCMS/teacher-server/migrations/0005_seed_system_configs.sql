-- 初始化 system_configs 种子数据
-- 用途：P0 阶段系统基础配置

INSERT INTO system_configs (id, config_key, config_value, scope, description, updated_at)
VALUES 
    (
        'cfg-heartbeat-interval',
        'heartbeat_interval_seconds',
        '15',
        'global',
        '学生端心跳上报间隔（秒）',
        datetime('now')
    ),
    (
        'cfg-default-mode',
        'default_mode',
        'open',
        'global',
        '默认课堂模式',
        datetime('now')
    ),
    (
        'cfg-teacher-fingerprint',
        'teacher_fingerprint',
        'pending_init',
        'global',
        '教师机指纹（首次启动时生成）',
        datetime('now')
    ),
    (
        'cfg-device-register-policy',
        'device_register_policy',
        'whitelist_optional',
        'global',
        '设备注册策略：whitelist_required | whitelist_optional',
        datetime('now')
    );
