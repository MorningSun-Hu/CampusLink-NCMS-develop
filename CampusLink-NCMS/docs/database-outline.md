# CampusLink-NCMS 数据库概要

## 核心表

- `students`
- `student_devices`
- `sign_records`
- `check_records`
- `hardware_snapshots`
- `process_policies`
- `process_alerts`
- `network_accounts`
- `system_configs`
- `operation_logs`

## 数据分层建议

- 开发环境使用 SQLite + SQLCipher
- 生产环境使用 PostgreSQL
- 教师端服务统一封装 Repository 层
- 学生端本地缓存与上报记录使用独立加密数据库
