# P10 执行基线：网络认证 + 设备发现 + 通信升级 + 安全加固

## 目标

补齐 V4.1 剩余全部缺口：网络认证管理(4.9)、UDP 广播发现、设备白名单、Protobuf 协议、SQLCipher 加密、维修工单、硬件检测。

## 任务拆解（10 项）

### T1. 网络认证管理 (JWT + 前端登录)

**文件**: `teacher-server/src/api/auth_handlers.rs`、`teacher-server/src/domain/auth.rs`、`teacher-server/src/middleware/auth.rs`

- `POST /api/auth/login`：验证用户名密码，返回 JWT token
- `POST /api/auth/refresh`：刷新 token
- Axum middleware：保护除 `/api/health`、`/api/auth/*`、`/ws` 之外的所有路由
- `teacher-web/src/views/Login.vue`：对接真实 API，存储 token 到 localStorage
- axios interceptor：自动附加 `Authorization: Bearer <token>`，401 自动跳转登录

### T2. UDP 广播设备发现

**文件**: `student-agent/agent-core/src/discovery.rs`

- 学生端启动时发送 UDP 广播包（端口 9999）
- 教师端监听 UDP，响应服务器地址和端口
- 学生端收到响应后自动配置 `teacher_server_url`
- 支持重试机制（最多 5 次，间隔 2s）

**教师端文件**: `teacher-server/src/discovery.rs`

- 启动 UDP 监听器（`tokio::net::UdpSocket`）
- 收到发现请求后回复 `{"host":"...", "port":..., "teacher_fingerprint":"..."}`

### T3. 设备白名单批量导入

**文件**: `teacher-server/src/domain/device.rs`、`teacher-web/src/views/DeviceWhitelist.vue`

- `device_whitelist` 表：`id, device_code (UNIQUE), device_name, mac_address, status(pending/approved), created_at`
- `POST /api/devices/whitelist/import`：Excel 批量导入
- `GET /api/devices/whitelist`：查询白名单
- `POST /api/devices/whitelist/:id/approve`：批准
- `DELETE /api/devices/whitelist/:id`：删除
- 设备注册时检查白名单：`whitelist_required` 模式下未在白名单则拒绝

### T4. AES256-GCM WebSocket 通信加密

**文件**: `teacher-server/src/crypto.rs`、`agent-core/src/crypto.rs`

- WebSocket 消息加密格式：`[12字节 nonce][密文]` (AES-256-GCM)
- 密钥协商：注册时教师端返回 `session_key`（256-bit 随机），学生端存储到 config
- 所有 WS 消息使用 session_key 加密（心跳、命令、状态上报）
- 发送侧：`encrypt(json_bytes, session_key) -> Vec<u8>` → `Message::Binary(vec)`
- 接收侧：`Message::Binary(vec)` → `decrypt(vec, session_key) -> json`

### T5. Protobuf 协议切换

**文件**: `agent-core/build.rs`、`teacher-server/build.rs`、proto 消息定义

- 添加 `build.rs` 编译 `proto/*.proto` 为 Rust 代码
- 替换 JSON 序列化为 Protobuf 编码（配合 T4 加密）
- 教师端和学生端共用 proto 定义
- 消息类型：`Heartbeat`、`ModeSwitch`、`LockScreen`、`Unlock`、`SuperPwd`、`AuthError`

### T6. SQLCipher 数据库加密

**文件**: `teacher-server/src/infrastructure/database.rs`、`teacher-server/Cargo.toml`

- 添加 `sqlx` SQLCipher feature 或直接使用 `libsqlite3-sys` + `sqlcipher`
- 启动时读取 `DB_KEY` 环境变量作为加密密钥
- 数据库连接添加 `PRAGMA key = '...'`
- 首次启动时若 db 文件为明文则自动加密迁移

### T7. 维修工单模块

**文件**: `teacher-server/src/domain/repair.rs`、`teacher-server/src/api/repair_handlers.rs`、migration

- `repair_orders` 表：`id, device_id, reporter, issue_type, description, status(pending/processing/completed), assigned_to, created_at, resolved_at`
- `POST /api/repair-orders`：创建工单
- `GET /api/repair-orders`：工单列表（筛选、分页）
- `PUT /api/repair-orders/:id`：更新状态
- `DELETE /api/repair-orders/:id`：删除
- 前端 `Repairs.vue`：工单管理页面

### T8. 键盘/鼠标自动检测上报

**文件**: `agent-core/src/hardware.rs`（扩展）

- 使用 `sysinfo` + 平台 API 检测 USB HID 设备插入/移除
- Windows：监听 `WM_DEVICECHANGE` 消息
- 检测到键盘/鼠标变更时生成事件 → WebSocket 上报 `hardware_change` 事件
- 教师端 `Hardware.vue` 实时展示外设变更

### T9. 前端补全

**文件**: `teacher-web/src/views/Repairs.vue`、`DeviceWhitelist.vue`、`Monitor.vue`（增强）

- 维修工单管理页面
- 设备白名单管理页面（批量导入/批准）
- Sidebar 新增菜单项 + 路由注册
- Dashboard 卡片增强：显示待处理工单数、白名单审核数

### T10. 全量编译验证 + 发布包更新

- `cargo build --release --target x86_64-pc-windows-gnu`（全部 crates）
- `npx vite build`（前端）
- Windows 发布包更新

## 交付物清单

| # | 文件 | 说明 |
|---|------|------|
| 1 | `teacher-server/src/api/auth_handlers.rs` | JWT 登录 API |
| 2 | `teacher-server/src/domain/auth.rs` | 认证业务逻辑 |
| 3 | `teacher-server/src/middleware/auth.rs` | Auth 中间件 |
| 4 | `teacher-server/src/discovery.rs` | UDP 监听响应 |
| 5 | `agent-core/src/discovery.rs` | UDP 广播发现 |
| 6 | `teacher-server/src/domain/device.rs` (扩展) | 白名单注册检查 |
| 7 | `teacher-server/src/api/repair_handlers.rs` | 维修工单 API |
| 8 | `teacher-server/src/domain/repair.rs` | 工单业务逻辑 |
| 9 | `teacher-server/src/crypto.rs` | WS 消息加解密 |
| 10 | `agent-core/src/crypto.rs` (扩展) | 客户端 WS 加解密 |
| 11 | `agent-core/build.rs` | Proto 编译 |
| 12 | `teacher-server/build.rs` | Proto 编译 |
| 13 | `migrations/...` | 白名单表、工单表 |
| 14 | `teacher-web/src/views/Repairs.vue` | 工单页面 |
| 15 | `teacher-web/src/views/DeviceWhitelist.vue` | 白名单页面 |
| 16 | `teacher-web/src/api/repairs.ts` | 工单 API |
| 17 | `teacher-web/src/api/auth.ts` | 登录 API |

## 验收标准

- [ ] `POST /api/auth/login` 返回有效 JWT token
- [ ] 未认证请求返回 401
- [ ] 前端登录流程完整（输入账号密码 → token → 跳转 dashboard）
- [ ] UDP 广播发现：学生端无需手动配置 URL，同一局域网自动发现
- [ ] 设备白名单导入：Excel 批量导入 → 审批 → 注册
- [ ] WS 消息加密后内容不可读（wireshark 抓包无法看到明文）
- [ ] proto 文件编译通过，所有消息使用 Protobuf 编码
- [ ] 数据库文件内容加密（hexdump 不可见明文数据）
- [ ] 维修工单 CRUD 完整
- [ ] 键盘/鼠标插拔事件实时上报
- [ ] 前端所有新增页面可用
