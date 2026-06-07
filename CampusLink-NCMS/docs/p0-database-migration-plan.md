# P0 数据库迁移清单

## 一、目标

为 P0 最小业务闭环提供可落库的数据结构，覆盖学生、设备、系统配置、操作日志四类核心数据。

## 二、迁移顺序

1. 创建 `students`
2. 创建 `student_devices`
3. 创建 `system_configs`
4. 创建 `operation_logs`
5. 补充索引与唯一约束
6. 写入初始化配置数据

## 三、表结构清单

### 1. `students`

用途：保存学生基础信息与登录凭据。

字段：

- `id`：UUID，主键
- `student_no`：字符串，学号，唯一
- `name`：字符串，学生姓名
- `password_hash`：字符串，密码哈希
- `seat_no`：字符串，座位号，可空
- `status`：字符串，状态，默认 `active`
- `created_at`：时间
- `updated_at`：时间

约束与索引：

- `student_no` 唯一索引
- `status` 普通索引

### 2. `student_devices`

用途：保存学生机注册信息、在线状态和当前模式。

字段：

- `id`：UUID，主键
- `student_id`：UUID，可空，关联 `students.id`
- `device_code`：字符串，设备编号，唯一
- `device_name`：字符串，设备名称
- `machine_fingerprint`：字符串，机器指纹，唯一
- `hostname`：字符串，主机名
- `ip_address`：字符串
- `mac_address`：字符串
- `register_status`：字符串，默认 `pending`
- `online_status`：字符串，默认 `offline`
- `last_seen_at`：时间，可空
- `current_mode`：字符串，默认 `open`
- `agent_version`：字符串
- `created_at`：时间
- `updated_at`：时间

约束与索引：

- `device_code` 唯一索引
- `machine_fingerprint` 唯一索引
- `student_id` 外键索引
- `online_status` 普通索引
- `last_seen_at` 普通索引

### 3. `system_configs`

用途：保存教师端全局配置与模式基础参数。

字段：

- `id`：UUID，主键
- `config_key`：字符串，唯一
- `config_value`：文本
- `scope`：字符串，默认 `global`
- `description`：字符串，可空
- `updated_at`：时间

约束与索引：

- `config_key` 唯一索引
- `scope` 普通索引

### 4. `operation_logs`

用途：记录注册、模式切换、系统操作等行为。

字段：

- `id`：UUID，主键
- `log_type`：字符串
- `operator`：字符串
- `target_id`：字符串，可空
- `content`：文本
- `extra_payload`：文本，可空
- `created_at`：时间

约束与索引：

- `log_type` 普通索引
- `target_id` 普通索引
- `created_at` 普通索引

## 四、初始化数据

### `system_configs` 初始化项

- `heartbeat_interval_seconds` = `15`
- `default_mode` = `open`
- `teacher_fingerprint` = `<启动时生成或首次写入>`
- `device_register_policy` = `whitelist_optional`

## 五、迁移文件建议

- `0001_create_students.sql`
- `0002_create_student_devices.sql`
- `0003_create_system_configs.sql`
- `0004_create_operation_logs.sql`
- `0005_seed_system_configs.sql`

## 六、编码落地要求

- 同时兼容 SQLite 与 PostgreSQL
- 时间字段统一使用 UTC
- UUID 生成策略在服务层统一封装
- 枚举值先使用字符串字段，P1 再决定是否收紧为数据库枚举
