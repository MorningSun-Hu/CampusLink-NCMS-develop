# 需求实施计划

- [x] 1. 写入会话超时系统配置（需求 2.1、2.2）
  - [x] 1.1 新增迁移 `CampusLink-NCMS/teacher-server/migrations/0020_usage_session_timeout.sql`
    - 向 `system_configs` 插入键 `usage_session_timeout_seconds`，值为 `60`，scope `global`
    - 描述写明：未结束使用会话的心跳超时秒数
  - [ ]* 1.2 确认 `sqlx::migrate!` 内存库可应用该迁移

- [x] 2. 实现心跳超时关闭未结束会话（需求 1.1-1.4、2.2、2.3、2.5）
  - [x] 2.1 在 `domain/usage.rs` 增加配置读取
    - 常量 `DEFAULT_USAGE_SESSION_TIMEOUT_SECONDS = 60` 与键名 `usage_session_timeout_seconds`
    - `usage_session_timeout_seconds(pool)`：缺失、无法解析或小于 1 时返回 60
  - [x] 2.2 在 `domain/usage.rs` 实现 `close_stale_sessions(pool)`
    - 关闭 `end_time IS NULL` 且设备 `last_seen_at` 早于当前时间减去阈值的会话
    - `end_time` 取设备 `last_seen_at`，`duration_seconds` 取 start_time 到 last_seen_at 且不小于 0
    - `last_seen_at` 为空的会话本轮跳过
  - [x] 2.3 在 `scheduler.rs` 每轮 `mark_stale_offline` 之后调用 `close_stale_sessions`
    - 每轮重新读配置，关闭行数大于 0 时打 info
  - [ ]* 2.4 编写 `close_stale_sessions` 单元测试
    - 超过阈值：会话关闭，end_time 等于 last_seen_at
    - 阈值内或 last_seen_at 为空：会话保持未结束
    - 配置缺失或非法按 60 秒；写入自定义正整数后按下值关闭

- [x] 3. 保持教师结束与下次签到路径（需求 3、4）
  - [x] 3.1 确认 `end_session` / `close_open_session` 仍写 `datetime('now')`
  - [x] 3.2 确认超时关闭后再 `start_session` 产生新记录且旧记录结束时间不变
  - [ ]* 3.3 为上述两条路径补充单元测试

- [x] 4. 检查点 - 确保所有测试通过,如有疑问请询问用户
