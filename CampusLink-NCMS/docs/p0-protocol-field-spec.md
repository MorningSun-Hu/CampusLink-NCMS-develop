# P0 协议字段定义

## 一、目标

为设备注册、心跳、模式切换三条实时链路建立可编码的 Protobuf 字段规范。

## 二、文件拆分

- `proto/common.proto`
- `proto/registration.proto`
- `proto/heartbeat.proto`
- `proto/mode.proto`

## 三、公共定义 `common.proto`

### 枚举 `ModeType`

- `MODE_OPEN = 0`
- `MODE_TEACHING = 1`
- `MODE_CONDITIONAL_OPEN = 2`
- `MODE_EXAM = 3`
- `MODE_LOCKED = 4`

### 枚举 `AckCode`

- `ACK_OK = 0`
- `ACK_REJECTED = 1`
- `ACK_INVALID = 2`
- `ACK_ERROR = 3`

### 消息 `MessageMeta`

- `string request_id`
- `int64 timestamp`
- `string protocol_version`

## 四、注册协议 `registration.proto`

### 消息 `RegisterRequest`

- `MessageMeta meta`
- `string device_code`
- `string machine_fingerprint`
- `string hostname`
- `string ip_address`
- `string mac_address`
- `string agent_version`

### 消息 `RegisterResponse`

- `MessageMeta meta`
- `AckCode code`
- `string message`
- `string device_id`
- `string teacher_fingerprint`
- `ModeType initial_mode`
- `uint32 heartbeat_interval_seconds`

## 五、心跳协议 `heartbeat.proto`

### 消息 `HeartbeatRequest`

- `MessageMeta meta`
- `string device_id`
- `ModeType current_mode`
- `string online_status`
- `string agent_version`

### 消息 `HeartbeatResponse`

- `MessageMeta meta`
- `AckCode code`
- `string message`
- `uint32 next_interval_seconds`

## 六、模式协议 `mode.proto`

### 消息 `ModeSwitchCommand`

- `MessageMeta meta`
- `string command_id`
- `string target_device_id`
- `ModeType target_mode`
- `int64 effective_at`
- `string operator_name`

### 消息 `ModeSwitchAck`

- `MessageMeta meta`
- `string command_id`
- `string device_id`
- `AckCode code`
- `string message`
- `ModeType applied_mode`

## 七、编码要求

- 每个消息都带 `meta`
- 时间字段统一为 Unix 秒级时间戳
- 设备模式统一走 `ModeType`
- `target_device_id` 为空时可扩展为广播场景，P0 先支持单设备
- 加密封装层放在传输适配器中，协议消息本身只定义明文字段
