# P9 执行基线：学生管理 + 锁屏完善 + 安全加固

## 目标

补齐 P0-P7 审计发现的功能缺口，重点覆盖学生信息管理（完整模块缺失）、锁屏安全（快捷键屏蔽）、本地加密与运维安全。

## 任务拆解（12 项）

### T1. 学生 CRUD REST API

**文件**: `teacher-server/src/api/student_handlers.rs`、`teacher-server/src/domain/student.rs`

| 端点 | 方法 | 说明 |
|------|------|------|
| `/api/students` | POST | 新增学生 |
| `/api/students` | GET | 学生列表（支持 search/keyword 搜索、分页） |
| `/api/students/:id` | GET | 学生详情 |
| `/api/students/:id` | PUT | 更新学生 |
| `/api/students/:id` | DELETE | 删除学生 |

请求/响应字段：`id`, `student_no`, `student_name`, `class_name`, `seat_no`, `password_hash`, `created_at`, `updated_at`。

### T2. 学生管理 Web 页面

**文件**: `teacher-web/src/views/Students.vue`、`teacher-web/src/api/students.ts`

- 表格展示（分页、搜索）
- 新增/编辑弹窗（表单验证）
- 删除确认
- 侧边栏新增菜单项 + 路由注册

### T3. 学生 Excel 导入导出

**依赖**: `calamine`（读取 xlsx）+ `rust_xlsxwriter`（写入 xlsx）

- `POST /api/students/import`：上传 xlsx，批量导入
- `GET /api/students/export`：导出全部学生为 xlsx
- 前端上传按钮 + 导出按钮

### T4. 超级密码修改 API + Web UI

**教师端**:
- `GET /api/settings/lock-password`：获取当前密码（仅返回是否已设置）
- `PUT /api/settings/lock-password`：修改密码 → 更新 `system_configs` 表 + WebSocket 广播 `super_pwd` 通知全部学生端更新

**学生端**:
- 新增 `WebSocketCommand::SuperPwd { password: String }` 枚举
- 收到后更新 `config.lock_password` 并 `config.save()`

**前端**: 设置页面新增锁屏密码修改卡片

### T5. campus-lock 系统快捷键屏蔽

**文件**: `campus-lock/src/main.rs`

在 eframe NativeOptions 基础上，通过 Win32 API 注册低级键盘钩子（`SetWindowsHookExW` with `WH_KEYBOARD_LL`）：

拦截键：
- `Win` (左/右)
- `Alt+Tab` / `Alt+F4`
- `Ctrl+Esc` / `Ctrl+Shift+Esc`（任务管理器）
- `Esc`（某些场景退出全屏）
- `F1`（帮助）

通过 `KBDLLHOOKSTRUCT` 判断组合键，返回 1 阻止传递。

需添加 `windows` crate feature: `Win32_UI_Input_KeyboardAndMouse`。

### T6. 本地配置 AES256-GCM 加密

**文件**: `agent-core/src/crypto.rs`

- 使用 `aes-gcm` + `rand` crate
- 密钥派生：从 `lock_password` 经 PBKDF2 派生 256-bit 密钥
- `config.json` → 写入时加密为 `config.enc`，读取时解密
- 初次启动时若 `config.enc` 不存在则从明文 `config.json` 迁移

### T7. 教师指纹双向校验

**文件**: `agent-core/src/register.rs`

- 注册时教师端返回 `teacher_fingerprint`
- 学生端保存到 config 的 `teacher_fingerprint`
- 每次 WebSocket 连接/心跳时携带 `teacher_fingerprint`
- 教师端验证：不匹配则拒绝连接并告警

### T8. 定时模式切换调度器

**文件**: `teacher-server/src/scheduler.rs`

- 使用 `tokio::time::interval` 实现
- 从 `system_configs` 表读取定时任务配置（schedule_mode, schedule_time, target_mode）
- 到时间后自动广播 `mode_switch` 到 WebSocket
- 支持每日定时（如每天 08:00 切授课模式，18:00 切开放模式）

### T9. SQLite 数据库每日备份

**文件**: `teacher-server/src/infrastructure/backup.rs`

- 启动时执行一次备份检查
- 每 24 小时定时备份 `data/campuslink.db` → `data/backup/YYYY-MM-DD/campuslink.db`
- 启动时清理超过 7 天的备份目录
- 使用 `tokio::spawn` + `std::fs::copy`

### T10. 日志轮转（tracing-appender）

**文件**: `teacher-server/src/main.rs`

- 添加 `tracing-appender` 依赖
- `RollingFileAppender::new("logs", RollingFileAppender::rotation::DAILY, "teacher-server")`
- 保留 14 天日志，超期自动删除
- Layer 组合：stdout + file

### T11. 双向进程守护

**文件**: `agent-core/src/process_guard.rs`

- 启动时检查 `campus-guard.exe` 是否运行
- 每 30 秒检查一次，不在则 `Command::new("campus-guard.exe").spawn()`
- 失败告警上报教师端

### T12. 编译验证与联调

- `cargo build --release --target x86_64-pc-windows-gnu`（全部 crates）
- `npx vite build`（前端）
- 更新 Windows 发布包

## 进度

- [x] T1 学生 CRUD API
- [x] T2 学生管理 Web 页面
- [x] T4 超级密码管理 (API + UI + WebSocket 广播)
- [x] T5 campus-lock 快捷键屏蔽 (WH_KEYBOARD_LL)
- [x] T6 AES256-GCM 配置加密 (config.enc)
- [x] T3 学生 Excel 导入导出
- [x] T7 教师指纹双向校验
- [x] T8 定时模式切换调度器
- [x] T9 SQLite 数据库每日备份
- [x] T10 日志轮转 (tracing-appender)
- [x] T11 双向进程守护
- [x] T12 全量编译验证 + 发布包更新

## 交付物清单

| # | 文件 | 说明 |
|---|------|------|
| 1 | `teacher-server/src/api/student_handlers.rs` | 学生 CRUD API |
| 2 | `teacher-server/src/domain/student.rs` | 学生业务逻辑 |
| 3 | `teacher-server/src/api/settings_handlers.rs` | 超级密码管理 API |
| 4 | `teacher-server/src/scheduler.rs` | 定时模式切换 |
| 5 | `teacher-server/src/infrastructure/backup.rs` | 数据库备份 |
| 6 | `teacher-server/Cargo.toml` | 新增 calamine, rust_xlsxwriter, tracing-appender |
| 7 | `teacher-web/src/views/Students.vue` | 学生管理页面 |
| 8 | `teacher-web/src/views/Settings.vue` | 设置页面（锁屏密码） |
| 9 | `teacher-web/src/api/students.ts` | 学生 API 封装 |
| 10 | `teacher-web/src/api/settings.ts` | 设置 API 封装 |
| 11 | `agent-core/src/crypto.rs` | AES256 加密工具 |
| 12 | `agent-core/src/command_handler.rs` | SuperPwd 命令处理 |
| 13 | `agent-core/src/process_guard.rs` | campus-guard 监控 |
| 14 | `agent-core/src/register.rs` | 指纹校验 |
| 15 | `campus-lock/src/main.rs` | 低级键盘钩子 |
| 16 | `campus-lock/Cargo.toml` | Win32_UI_Input_KeyboardAndMouse feature |
| 17 | `agent-core/Cargo.toml` | aes-gcm, rand |

## 验收标准

- [ ] `POST /api/students` 可创建学生
- [ ] `GET /api/students` 分页返回 + 关键词搜索
- [ ] `PUT /api/students/:id` 更新学生信息
- [ ] `DELETE /api/students/:id` 删除学生
- [ ] 前端 Students.vue 完整 CRUD 操作
- [ ] Excel 批量导入成功
- [ ] Excel 导出下载成功
- [ ] 超级密码修改后学生端实时更新
- [ ] 锁屏状态下 Win/Alt+Tab/Alt+F4/Ctrl+Esc 全部屏蔽
- [ ] config.enc 文件加密存储
- [ ] 教师指纹不匹配时 WebSocket 被拒绝
- [ ] 定时模式切换按配置时间自动执行
- [ ] 数据库备份文件存在于 backup/YYYY-MM-DD/
- [ ] 日志文件按天轮转，过期自动删除
- [ ] campus-guard 异常退出后被 agent-core 重新拉起
