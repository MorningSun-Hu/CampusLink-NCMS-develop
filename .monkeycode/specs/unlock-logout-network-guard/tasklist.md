# 需求实施计划

 - [x] 1. 超级密码解锁后同步教师端模式（需求 1）
   - [x] 1.1 新增 `POST /api/devices/local-unlock`
    - 请求体含 `device_id`，校验设备已注册后调用 `unlock_restore_mode`
    - 不向该设备再广播 `unlock`，不关闭使用会话
    - 在 `api/mod.rs` 注册为公开路由
   - [x] 1.2 JSON 心跳补同步模式（需求 1.2）
    - `process_heartbeat_json` 读取心跳 `mode`
    - 库中为 `locked` 且心跳为 `open`/`teaching`/`exam` 时走 `unlock_restore_mode`
   - [x] 1.3 学生机解锁后上报
    - `campus-lock` 退出的 `unlock_rx` 路径在 `apply_unlock_restore` 之后 POST `local-unlock`
    - 签到窗超级密码解锁路径同样上报
  - [ ]* 1.4 编写解锁同步单元测试
    - locked + mode_before_lock=teaching → current_mode=teaching
    - 使用会话 end_time 保持为空

 - [x] 2. 教师端退出登录（需求 2）
   - [x] 2.1 `Layout.vue` 页眉增加「退出」
    - 断开教师端 WebSocket，清除 `token` 与 `user`，跳转 `/login`

 - [x] 3. 授课模式网络异常锁定与恢复告警（需求 3、4、5）
   - [x] 3.1 新增 `agent-core/src/netguard.rs` 相邻问询
    - 监听 UDP 9998，应答 `teacher_reachable`
    - 广播 `CAMPUSLINK_PEER_PING`，2 秒超时、最多重试 1 次
   - [x] 3.2 实现物理链路检测与判定函数
    - 链路断开 → `physical_link`
    - 链路正常、教师不可达、邻居 `teacher_reachable=true` → `firewall_block`
    - 邻居全 false 或两次无应答 → 保持授课
   - [x] 3.3 在心跳循环接入锁定与恢复
    - 授课模式 1 秒检查；3 秒内 overlay 拉起 `campus-lock`
    - 网络恢复后恢复授课、不重新签到、不结束使用会话
   - [x] 3.4 恢复后写告警并推送教师端
    - `inspection_type=network`，`item_name` 为 `physical_link` 或 `firewall_block`
    - `submit_inspection` 在异常时广播 WS `type=alert`
    - `Alerts.vue` 与 `useWebSocket` 增加网络告警文案
   - [x]* 3.5 编写 netguard 判定单元测试
    - 覆盖物理断开、故意阻断、教师机故障三条路径

 - [x] 4. 检查点 - 确保所有测试通过,如有疑问请询问用户
