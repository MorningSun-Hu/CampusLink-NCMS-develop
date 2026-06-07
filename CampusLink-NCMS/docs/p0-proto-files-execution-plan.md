# P0 协议 .proto 文件执行基线文档

## 一、目标

本执行基线用于指导 `proto/` 目录下的 P0 协议 `.proto` 文件生成工作，确保注册、心跳、模式切换三条实时链路具备统一、可生成、可扩展的协议定义。

## 二、本次产出范围

本轮只生成 `.proto` 协议文件和基础生成约束，不编写消息处理代码。

目标文件：

- `proto/common.proto`
- `proto/registration.proto`
- `proto/heartbeat.proto`
- `proto/mode.proto`

## 三、生成顺序

1. 先生成 `common.proto`
2. 再生成 `registration.proto`
3. 再生成 `heartbeat.proto`
4. 最后生成 `mode.proto`

## 四、文件结构要求

### 通用要求

- 使用 `proto3` 语法
- 所有文件声明统一包名，建议 `campuslink.ncms.v1`
- 共享枚举和共享消息只放在 `common.proto`
- 其他文件通过 `import` 复用公共定义
- 字段编号从 `1` 开始连续编号
- 字段命名统一使用 `snake_case`

### 1. `common.proto`

必须包含：

- `ModeType`
- `AckCode`
- `MessageMeta`

字段要求：

- `MessageMeta.request_id = 1`
- `MessageMeta.timestamp = 2`
- `MessageMeta.protocol_version = 3`

### 2. `registration.proto`

必须包含：

- `RegisterRequest`
- `RegisterResponse`

字段顺序要求：

#### `RegisterRequest`

- `meta = 1`
- `device_code = 2`
- `machine_fingerprint = 3`
- `hostname = 4`
- `ip_address = 5`
- `mac_address = 6`
- `agent_version = 7`

#### `RegisterResponse`

- `meta = 1`
- `code = 2`
- `message = 3`
- `device_id = 4`
- `teacher_fingerprint = 5`
- `initial_mode = 6`
- `heartbeat_interval_seconds = 7`

### 3. `heartbeat.proto`

必须包含：

- `HeartbeatRequest`
- `HeartbeatResponse`

字段顺序要求：

#### `HeartbeatRequest`

- `meta = 1`
- `device_id = 2`
- `current_mode = 3`
- `online_status = 4`
- `agent_version = 5`

#### `HeartbeatResponse`

- `meta = 1`
- `code = 2`
- `message = 3`
- `next_interval_seconds = 4`

### 4. `mode.proto`

必须包含：

- `ModeSwitchCommand`
- `ModeSwitchAck`

字段顺序要求：

#### `ModeSwitchCommand`

- `meta = 1`
- `command_id = 2`
- `target_device_id = 3`
- `target_mode = 4`
- `effective_at = 5`
- `operator_name = 6`

#### `ModeSwitchAck`

- `meta = 1`
- `command_id = 2`
- `device_id = 3`
- `code = 4`
- `message = 5`
- `applied_mode = 6`

## 五、编码规范

- 每个消息都必须带 `MessageMeta`
- 所有时间字段使用 Unix 秒级时间戳
- `ModeType` 统一承载设备模式
- `AckCode` 统一承载成功与失败状态
- 当前阶段只定义消息结构，不定义 gRPC service
- 广播能力放到后续阶段，P0 先按单设备模式设计

## 六、生成验证

- 4 个 `.proto` 文件全部生成在 `proto/`
- 文件之间 `import` 关系清晰
- 字段编号无重复、无跳号
- 枚举与消息命名与 P0 协议文档一致
- 协议文件结构适合后续生成 Rust 与 TypeScript 代码

## 七、后续依赖

- `teacher-server` WebSocket 消息解析依赖这些协议文件
- `student-agent` 注册、心跳、模式切换依赖这些协议文件
- 前后端文档字段以 `.proto` 为最终准绳
