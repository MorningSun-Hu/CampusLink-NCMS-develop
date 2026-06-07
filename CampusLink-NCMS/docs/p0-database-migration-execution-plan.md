# P0 数据库迁移文件执行基线文档

## 一、目标

本执行基线用于指导 `teacher-server/migrations/` 目录下的 P0 数据库迁移文件生成工作，确保迁移文件可以直接支撑注册、心跳、在线状态与模式控制最小业务闭环。

## 二、本次产出范围

本轮只生成迁移文件与初始化种子文件，不实现 Repository、Service、Handler 代码。

目标文件：

- `teacher-server/migrations/0001_create_students.sql`
- `teacher-server/migrations/0002_create_student_devices.sql`
- `teacher-server/migrations/0003_create_system_configs.sql`
- `teacher-server/migrations/0004_create_operation_logs.sql`
- `teacher-server/migrations/0005_seed_system_configs.sql`

## 三、生成顺序

1. 先创建 `students`
2. 再创建 `student_devices`
3. 再创建 `system_configs`
4. 再创建 `operation_logs`
5. 最后写入 `system_configs` 初始化数据

## 四、迁移文件设计要求

### 1. 通用要求

- SQL 语句兼容 SQLite 开发环境
- 为 PostgreSQL 预留可迁移字段命名与类型语义
- 所有时间字段使用 UTC 语义
- 当前阶段主键字段统一使用 `TEXT` 或等价字符串类型承载 UUID
- 枚举先使用 `TEXT` 字段保存字符串值

### 2. `0001_create_students.sql`

必须创建：

- 主键 `id`
- 唯一学号 `student_no`
- 姓名 `name`
- 密码哈希 `password_hash`
- 座位号 `seat_no`
- 状态 `status`
- 创建时间 `created_at`
- 更新时间 `updated_at`

必须包含：

- `student_no` 唯一约束
- `status` 索引

### 3. `0002_create_student_devices.sql`

必须创建：

- 主键 `id`
- 外键 `student_id`
- 唯一设备编号 `device_code`
- 设备名称 `device_name`
- 唯一机器指纹 `machine_fingerprint`
- 主机名 `hostname`
- IP 地址 `ip_address`
- MAC 地址 `mac_address`
- 注册状态 `register_status`
- 在线状态 `online_status`
- 最后心跳时间 `last_seen_at`
- 当前模式 `current_mode`
- Agent 版本 `agent_version`
- 创建时间 `created_at`
- 更新时间 `updated_at`

必须包含：

- `students(id)` 外键关联
- `device_code` 唯一约束
- `machine_fingerprint` 唯一约束
- `student_id` 索引
- `online_status` 索引
- `last_seen_at` 索引

### 4. `0003_create_system_configs.sql`

必须创建：

- 主键 `id`
- 配置键 `config_key`
- 配置值 `config_value`
- 作用域 `scope`
- 描述 `description`
- 更新时间 `updated_at`

必须包含：

- `config_key` 唯一约束
- `scope` 索引

### 5. `0004_create_operation_logs.sql`

必须创建：

- 主键 `id`
- 日志类型 `log_type`
- 操作人 `operator`
- 目标对象 `target_id`
- 日志内容 `content`
- 扩展载荷 `extra_payload`
- 创建时间 `created_at`

必须包含：

- `log_type` 索引
- `target_id` 索引
- `created_at` 索引

### 6. `0005_seed_system_configs.sql`

必须写入：

- `heartbeat_interval_seconds`
- `default_mode`
- `teacher_fingerprint`
- `device_register_policy`

初始化值要求：

- `heartbeat_interval_seconds` = `15`
- `default_mode` = `open`
- `teacher_fingerprint` = `pending_init`
- `device_register_policy` = `whitelist_optional`

## 五、文件内容规范

- 每个文件只处理一个明确主题
- 先写 `CREATE TABLE`，再写索引
- 种子文件只写初始化 `INSERT`
- SQL 注释使用单行注释写在语句上方
- 不在同一文件混入多类资源创建逻辑

## 六、验证标准

- 迁移文件命名与顺序正确
- 5 个文件全部生成在 `teacher-server/migrations/`
- 表字段、索引、唯一约束与 P0 清单一致
- 初始化配置项完整
- 迁移文件具备直接进入编码阶段的清晰结构

## 七、后续依赖

- 数据库接入代码实现时，按这些迁移文件建立模型映射
- 后端接口开发时，优先依赖 `students`、`student_devices`、`system_configs`、`operation_logs`
