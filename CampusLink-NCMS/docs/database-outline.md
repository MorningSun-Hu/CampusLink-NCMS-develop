# CampusLink-NCMS 数据库概要

当前实现使用 SQLite（WAL），教师端启动时自动执行嵌入式迁移。生产 PostgreSQL / SQLCipher 为预留能力。

## 核心表（按迁移）

| 表 | 迁移 | 用途 |
|----|------|------|
| `students` | 0001 / 0017 | 学生；班级、改密标记、座位 |
| `student_devices` | 0002 / 0017 / 0019 | 设备；班级座位、`mode_before_lock` |
| `system_configs` | 0003 / 0005 | 系统配置 |
| `operation_logs` | 0004 | 操作审计 |
| `attendance_records` | 0006 / 0017 / 0018 | 授课考勤；座位快照、使用会话关联 |
| `inspection_records` | 0007 | 环境/设备检查 |
| `photos` | 0008 | 检查照片 |
| `hardware_snapshots` | 0009 | 硬件快照 |
| `hardware_changes` | 0010 | 硬件变更 |
| `process_guard_policies` | 0011 | 进程守护策略 |
| `admin_users` | 0012 | 教师账号 |
| `device_whitelist` | 0013 | 设备白名单 |
| `repair_orders` | 0014 | 维修工单 |
| `network_accounts` | 0015 | 网络认证账号 |
| `classes` | 0016 | 班级 |
| `device_usage_records` | 0018 | 设备使用会话（开放模式签到走此表） |

完整字段以 `teacher-server/migrations/` 为准。阶段说明见 `docs/p12-stage-summary.md`。
