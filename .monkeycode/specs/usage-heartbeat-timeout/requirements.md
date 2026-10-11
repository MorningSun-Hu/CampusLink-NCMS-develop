# Requirements Document

## Introduction

补齐设备使用记录在「教师未点结束会话、学生机关机或断连」时的结束时间。当前未结束会话的 `end_time` 会一直为空。本需求规定：心跳超时后自动结束会话，结束时间取该设备最后一次心跳时间。

涉及范围：`teacher-server`（调度器、使用记录领域）。教师端使用查询沿用现有列表与导出。

## Glossary

- **使用会话**：`device_usage_records` 中一条从签到开始到结束的设备使用记录。
- **未结束会话**：`end_time` 为空的使用会话。
- **最后心跳时间**：`student_devices.last_seen_at`，由学生机心跳写入。
- **心跳超时**：自最后心跳时间起，连续超过会话超时阈值仍无新心跳。
- **会话超时阈值**：判定未结束会话应自动结束的等待时长，来自系统配置 `usage_session_timeout_seconds`，缺省 60 秒。
- **教师结束会话**：教师在使用记录或监控入口主动关闭当前使用会话。
- **在线判定阈值**：监控将设备标为离线的心跳间隔倍数（现为心跳间隔的 3 倍，默认 45 秒），与会话超时阈值分开。

## Requirements

### Requirement 1: 心跳超时结束未结束会话

**User Story:** AS 教师, I want 学生机关机后使用记录仍有结束时间, so that 使用时长可统计。

#### Acceptance Criteria

1. WHEN 一条使用会话的 `end_time` 为空，并且该设备的最后心跳时间早于当前时间减去会话超时阈值, 系统 SHALL 关闭该使用会话。
2. WHEN 系统因心跳超时关闭使用会话, 系统 SHALL 将该会话的 `end_time` 设为该设备的最后心跳时间。
3. WHEN 系统因心跳超时关闭使用会话, 系统 SHALL 将 `duration_seconds` 设为开始时间到最后心跳时间的秒数，取值大于或等于 0。
4. IF 设备不存在最后心跳时间, 系统 SHALL 保持该未结束会话不变，等待下一次扫描。

### Requirement 2: 会话超时阈值

**User Story:** AS 机房管理员, I want 短暂断网不要立刻结掉使用记录, so that 一次使用不会被拆成多条。

#### Acceptance Criteria

1. 系统 SHALL 从 `system_configs` 读取键 `usage_session_timeout_seconds` 作为会话超时阈值。
2. IF 该配置缺失、无法解析、或解析结果小于 1, 系统 SHALL 使用 60 秒作为会话超时阈值。
3. 系统 SHALL 将会话超时阈值与在线判定阈值分开计算。
4. WHILE 教师服务器进程运行, 系统 SHALL 至少每 30 秒扫描一次未结束会话并按 Requirement 1 处理。
5. WHEN 管理员更新 `usage_session_timeout_seconds`, 系统 SHALL 在下一次扫描使用新阈值。

### Requirement 3: 教师主动结束仍用当前时间

**User Story:** AS 教师, I want 我点结束会话时立刻记下结束时间, so that 主动下课与关机超时可区分。

#### Acceptance Criteria

1. WHEN 教师主动结束某设备的使用会话, 系统 SHALL 将该会话的 `end_time` 设为当前 UTC 时间。
2. WHEN 教师主动结束某设备的使用会话, 系统 SHALL 将 `duration_seconds` 设为开始时间到当前 UTC 时间的秒数。

### Requirement 4: 超时结束后的下一次使用

**User Story:** AS 教师, I want 关机后再开机的使用记成新会话, so that 两次上机时长分开统计。

#### Acceptance Criteria

1. WHEN 设备在心跳超时关闭会话之后重新连上教师机, 系统 SHALL 保持已关闭会话的结束时间不变。
2. WHEN 该设备下一次签到成功, 系统 SHALL 开启一条新的使用会话，开始时间为本次签到时间。

## 已确认决策

1. 心跳超时关闭会话时，结束时间取 `last_seen_at`，不用扫描时刻。
2. 会话超时阈值写入 `system_configs.usage_session_timeout_seconds`，缺省 60 秒，与监控离线（约 45 秒）分开。
3. 教师主动结束会话仍写当前时间。
4. 超时关闭后，等下次签到再开新会话。
5. 使用记录列表只写结束时间，不加结束原因字段。
