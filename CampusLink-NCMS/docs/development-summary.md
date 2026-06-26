# CampusLink-NCMS 开发总结报告

## 一、项目概述

CampusLink-NCMS 是一个网络教室使用管理系统，包含教师端服务、教师端 Web 管理界面、学生端原生 Agent、实时通信协议与 Windows 发布资源。

技术栈：
- 教师端服务：Rust + Axum + SQLx + SQLite
- 教师端 Web：Vue 3 + TypeScript + Element Plus + Vite
- 学生端 Agent：Rust + Tokio + WebSocket
- 通信协议：当前 Windows 联调包使用 JSON over WebSocket，Protobuf 协议文件已预留

## 二、当前开发进度

| 阶段 | 任务 | 状态 | 说明 |
|------|------|------|------|
| P0 | 数据库迁移、协议文件、教师端基础服务 | 完成 | 表结构和基础服务已落地 |
| P1 | 设备注册与心跳链路（教师端） | 完成 | Windows 实机注册已验证 |
| P2 | 教师端 Web 基础页面 | 完成 | 基础页面与 API 封装已落地 |
| P3 | 学生端 Agent 注册与心跳 | 完成 | Windows 学生端可注册并进入心跳循环 |
| P4 | 模式切换基础链路 | 完成 | REST API 与 WebSocket 广播链路已落地 |
| P5 | WebSocket 心跳循环与锁屏框架 | 完成 | 30 秒心跳、断线重连、锁屏框架已落地 |
| P6 | 签到与检查流程 | 完成 | API + 学生端 + 前端全链路已实现，待实机联调 |
| P7 | 硬件快照、日志中心、进程守护与锁屏完善 | 完成 | 3 张新表、7 个 API、2 个学生端模块、2 个前端页面、锁屏全链路实现与联调修复 |
| P8 | 部署与安全加固 | 待开始 | Windows 服务注册、安装器打包、域名/HTTPS、生产部署说明 |

## 三、本轮 Windows 联调确认结果

本轮完成了 Windows 发布包的教师端与学生端实机联调，验证结果如下：

- 教师端 `teacher-server.exe` 可在 Windows 桌面解压目录直接启动。
- 教师端启动脚本 `teacher-server/start.bat` 会基于脚本所在目录生成 SQLite 数据库路径。
- 教师端 SQLite 数据库文件可自动创建。
- 教师端启动时会自动执行 `migrations/` 中嵌入的数据库迁移，修复首次启动缺表问题。
- 学生端 `agent-core.exe` 可读取配置并完成设备注册。
- 学生端可建立 WebSocket 连接：`ws://localhost:8080/ws?device_id=...`。
- 学生端 30 秒心跳可发送，教师端可更新设备心跳状态。
- 教师端 WebSocket 心跳响应已统一为 `heartbeat_ack`，修复学生端 `unknown variant ack` 警告。

已验证日志关键点：

```text
Configuration loaded successfully
Database connection pool created
Database migrations applied
Starting server on 0.0.0.0:8080
Device registered successfully
WebSocket connected
Heartbeat sent
Received message: {"type":"heartbeat_ack","ack_code":0,"message":"ok"}
```

## 四、本轮代码与发布包更新

### P6 新增：签到与检查流程

#### 教师端服务

- 数据库迁移 0006-0008：`attendance_records`、`inspection_records`、`photos` 三张新表
- `teacher-server/src/api/attendance_handlers.rs`：签到 API（check-in、retroactive、statistics、list）
- `teacher-server/src/api/inspection_handlers.rs`：检查/告警 API（submit、list、alerts、resolve）
- `teacher-server/src/api/photo_handlers.rs`：图片上传 API（multipart upload、get、list）
- `teacher-server/src/domain/attendance.rs`：签到业务逻辑
- `teacher-server/src/domain/inspection.rs`：检查与告警业务逻辑
- `teacher-server/src/domain/photo.rs`：图片存储逻辑
- `teacher-server/Cargo.toml`：axum 新增 `multipart` feature

#### 学生端 Agent

- `agent-core/src/attendance.rs`：签到请求与自动签到
- `agent-core/src/inspection.rs`：检查提交与异常报告
- `agent-core/src/student_auth.rs`：登录/登出/状态持久化
- `agent-core/src/config.rs`：扩展学生身份字段（student_id、token 等）
- `agent-core/src/main.rs`：启动时自动执行签到

#### 教师端 Web

- `src/views/Attendance.vue`：签到管理页（记录列表、统计、补签弹窗）
- `src/views/Alerts.vue`：检查告警页（检查记录、告警处理、图片上传与预览）
- `src/api/attendance.ts`：签到 API 封装
- `src/api/inspection.ts`：检查/告警 API 封装
- `src/api/photos.ts`：图片上传 API 封装
- `src/router/index.ts`：新增 /attendance、/alerts 路由
- `src/components/Layout.vue`：侧边栏新增签到管理、检查告警

### P0-P5 发布包修复（历史）

### 教师端服务

- `teacher-server/src/infrastructure/database.rs`
  - 使用 `SqliteConnectOptions::create_if_missing(true)` 自动创建 SQLite 数据库文件。
  - 连接数据库前自动创建父目录。
  - 兼容 `sqlite:///C:/...`、`sqlite://...`、`sqlite:...` 形式的 SQLite URL。

- `teacher-server/src/main.rs`
  - 启动时执行 `sqlx::migrate!("./migrations")`。
  - 迁移文件嵌入可执行文件，Windows 发布包无需额外复制迁移目录。

- `teacher-server/src/api/handlers.rs`
  - WebSocket 心跳响应从 `{"type":"ack"}` 调整为 `{"type":"heartbeat_ack","ack_code":0,"message":"ok"}`。

### Windows 发布包

- `dist/windows-release/teacher-server.exe`
  - 已替换为最新编译产物。

- `dist/windows-release/teacher-server/teacher-server.exe`
  - 已替换为最新编译产物。

- `dist/windows-release/teacher-server/start.bat`
  - 使用当前目录生成数据库路径。
  - 使用 ASCII 内容，避免 Windows CMD 中文编码乱码。

- `dist/windows-release/install.bat`
  - 使用 ASCII 内容，避免 Windows CMD 中文编码乱码。

- `dist/windows-release/student-agent/start.bat`
  - 使用 ASCII 内容，避免 Windows CMD 中文编码乱码。

## 五、当前已完成功能清单

### 1. 数据库层

表结构：
- `students`：学生信息表
- `student_devices`：学生机设备表
- `system_configs`：系统配置表
- `operation_logs`：操作日志表

迁移文件：
- `0001_create_students.sql`
- `0002_create_student_devices.sql`
- `0003_create_system_configs.sql`
- `0004_create_operation_logs.sql`
- `0005_seed_system_configs.sql`
- `0006_create_attendance_records.sql`
- `0007_create_inspection_records.sql`
- `0008_create_photos.sql`
- `0009_create_hardware_snapshots.sql`
- `0010_create_hardware_changes.sql`
- `0011_create_process_guard_policies.sql`

### 2. 教师端服务

接口与能力：
- `GET /api/health`：健康检查
- `GET /api/dashboard/overview`：仪表盘统计概览
- `POST /api/devices/register`：设备注册
- `POST /api/devices`：设备列表查询
- `POST /api/devices/:id/mode`：模式切换
- `POST /api/devices/:id/lock`：远程锁屏（WebSocket 广播 lock_screen 命令）
- `POST /api/devices/:id/unlock`：远程解锁（WebSocket 广播 unlock 命令，taskkill）
- `POST /api/attendance/check-in`：学生签到
- `POST /api/attendance/retroactive`：教师补签
- `GET /api/attendance`：签到记录列表
- `GET /api/attendance/statistics`：签到统计
- `GET /api/inspection`：检查记录列表
- `POST /api/alerts/:id/resolve`：处理告警
- `GET /api/alerts`：告警列表
- `POST /api/photos/upload`：图片上传（multipart）
- `GET /api/photos`：图片列表（可选 `inspection_id` 筛选）
- `GET /api/photos/:id`：图片下载预览
- `POST /api/hardware/snapshot`：硬件快照提交
- `GET /api/hardware/snapshot`：设备硬件快照查询
- `GET /api/hardware/changes`：硬件变更记录列表
- `GET /api/logs`：操作日志查询（分页、类型筛选）
- `POST /api/policies/process-guard`：创建进程守护策略
- `GET /api/policies/process-guard`：查询进程守护策略列表
- `DELETE /api/policies/process-guard/:id`：删除进程守护策略
- `/ws`：WebSocket 实时通信入口
- 自动创建 SQLite 数据库文件
- 启动时自动执行数据库迁移
- 接收学生端心跳并返回 `heartbeat_ack`
- 内嵌前端静态文件（SPA fallback，端口 8080 一体化部署）

### 3. 学生端 Agent

主进程能力：
- 本机信息采集（主机名、IP、MAC、机器指纹）
- 首次启动自动注册设备
- 配置持久化
- WebSocket 连接与 30 秒心跳循环
- 收到 `heartbeat_ack` 后正常处理
- 启动时自动签到
- 检查提交与异常报告（卫生检查、设备检查）
- 学生登录与登录状态持久化
- 硬件信息采集与上报（CPU、内存、磁盘、网卡、OS、GPU）
- 进程守护策略同步与进程扫描告警
- 模式切换联动锁屏（locked/exam 模式自动启动 campus-lock，open 模式自动杀进程）
- 锁屏进程退出实时检测与状态上报（mpsc channel → 心跳循环 → WebSocket 通知教师端）

辅助进程：
- `campus-guard`：双进程守护（监控 agent-core 与 campus-lock），异常退出自动拉起
- `campus-lock`：全屏锁定 + 超级密码解锁（失败计数限流）

### 4. 教师端 Web

页面：
- `/login`：登录页占位
- `/dashboard`：仪表盘
- `/devices`：设备列表与模式切换弹窗
- `/monitor`：监控总览
- `/attendance`：签到管理（记录列表、统计卡片、补签）
- `/alerts`：检查告警（检查记录、告警处理、图片上传与预览）
- `/hardware`：硬件快照（设备硬件详情 + 变更记录表）
- `/logs`：日志中心（分页查询 + 类型筛选）

## 六、Windows 发布包状态

发布包目录：

```text
dist/windows-release/
├─ install.bat
├─ README-windows.md
├─ teacher-server.exe
├─ teacher-server/
│  ├─ start.bat
│  └─ teacher-server.exe
└─ student-agent/
   ├─ start.bat
   ├─ config.json
   ├─ agent-core.exe
   ├─ campus-guard.exe
   └─ campus-lock.exe
```

Windows 验证顺序：

```powershell
cd C:\Users\Hcy\Desktop\windows-release\windows-release\teacher-server
.\start.bat
```

```powershell
cd C:\Users\Hcy\Desktop\windows-release\windows-release\student-agent
.\start.bat
```

## 七、P7 变更记录

### P7 教师端新增（7 个 API 路由）

| 路由 | 方法 | 说明 |
|------|------|------|
| `/api/hardware/snapshot` | POST | 提交硬件快照（含变更检测） |
| `/api/hardware/snapshot` | GET | 查询设备最新硬件快照 |
| `/api/hardware/changes` | GET | 硬件变更记录列表 |
| `/api/logs` | GET | 日志查询（分页+类型/设备筛选） |
| `/api/policies/process-guard` | POST | 创建进程守护策略 |
| `/api/policies/process-guard` | GET | 查询进程守护策略列表 |
| `/api/policies/process-guard/:id` | DELETE | 删除进程守护策略 |

### P7 数据库迁移（3 张新表）

- `0009_create_hardware_snapshots.sql`：硬件快照（CPU、内存、磁盘、网卡、GPU、OS）
- `0010_create_hardware_changes.sql`：硬件变更检测记录
- `0011_create_process_guard_policies.sql`：进程守护策略（支持全局/设备绑定）

### P7 学生端新增模块

- `agent-core/src/hardware.rs`：sysinfo 采集 + 启动时自动上报
- `agent-core/src/process_guard.rs`：策略同步 + 进程列表扫描 + 告警生成
- `campus-lock/src/locker.rs`：锁屏密码验证 + 失败计数限流
- `campus-guard`：增强为双进程守护（agent-core + campus-lock），异常退出计数上限 10

### P7 前端新增

- `src/views/Hardware.vue`：硬件快照详情 + 变更记录表
- `src/views/Logs.vue`：日志分页列表 + 类型/设备筛选
- `src/api/hardware.ts`、`src/api/logs.ts`：API 封装
- 侧边栏新增 2 个菜单项

### P7 锁屏全链路（补充）

#### 触发链路

```
教师端 Web [锁屏按钮] 或 [切换模式→锁定]
  → POST /api/devices/:id/mode  (mode="locked"/"exam")
  → 教师端 WebSocket 广播 {"type":"mode_switch","mode":"locked",...}
  → 学生端 command_handler 接收 → mode.rs lock_completely()
  → spawn campus-lock.exe (全屏 egui GUI)
  → campus-lock 输入超级密码解锁 / 教师端远程 taskkill
```

#### 教师端新增

- `src/api/handlers.rs`：`lock_screen_handler` + `unlock_handler`（广播 `lock_screen`/`unlock` 命令）
- `src/api/mod.rs`：路由注册 `POST /api/devices/:id/lock`、`POST /api/devices/:id/unlock`
- `src/domain/device.rs`：新增 `update_device_mode()` 轻量更新函数
- WebSocket 收到心跳时同步更新 `current_mode` 到数据库

#### 学生端新增/修改

- `agent-core/src/mode.rs`：`lock_completely()` / `enable_exam_mode()` 实际 spawn campus-lock.exe；`disable_all_locks()` 调用 taskkill 结束锁屏进程
- `agent-core/src/mode.rs`：spawn 后启动 `spawn_blocking` 监控进程退出，退出时通过 `mpsc::UnboundedSender` 通知心跳循环
- `agent-core/src/command_handler.rs`：持有 `unlock_tx` 传递给 mode handler；LockScreen/Unlock 枚举处理；使用 `current_exe().parent()` 定位 campus-lock.exe
- `agent-core/src/websocket.rs`：心跳循环收到解锁通知后立即更新 mode→"open"、保存 config、发送心跳上报教师端；收到命令后同步 command_handler 配置到心跳循环
- `agent-core/src/config.rs`：新增 `lock_password`/`is_locked`/`lock_pid` 字段

#### campus-lock 修复

- `campus-lock/src/main.rs`：移除 `ctx.request_repaint()` 持续重绘（修复 TextEdit 输入光标异常）
- `campus-lock/src/main.rs`：启动时 `ImmDisableIME(GetCurrentThreadId())` 禁用 Windows IME（修复输入法拦截键击导致字符重复/异常）
- `campus-lock/src/main.rs`：修复 Enter 键解锁逻辑，添加 stderr 错误日志输出
- `campus-lock/Cargo.toml`：新增 `Win32_UI_Input_Ime`、`Win32_System_Threading` features

#### 教师端 Web 新增

- `src/api/devices.ts`：`lockDevice()` / `unlockDevice()` API 封装
- `src/views/Devices.vue`：操作列新增锁屏/解锁按钮

### P7 Bug 修复

- **Layout.vue 导航失效**：`el-menu` 的 `router` 模式与 `default-active` 存在竞态，改用 `router-link` + `custom` v-slot 实现侧边栏
- **Attendance.vue 渲染错误**：后端统计 API 返回对象但 `el-table :data` 期望数组，改为卡片布局展示统计数据
- **签到数据字段对齐**：前端 `AttendanceRecord` 字段名对齐后端 snake_case（`check_in_time`/`check_out_time`/`status`）
- **仪表盘全为 0**：后端缺少 `/api/dashboard/overview` 端点，新增 `dashboard_handlers.rs` 实现统计查询
- **设备信息字段为空**：后端 `DeviceResponse` 序列化为 snake_case，前端期望 camelCase，添加 `#[serde(rename_all = "camelCase")]`
- **模式切换学生端无反应**：WebSocket 消息字段不匹配（后端 `target_mode` vs 前端/学生端 `mode`），统一为 `{"type":"mode_switch","device_id":"...","mode":"...","operator":"...","timestamp":...}`
- **学生端接收全部设备命令**：命令广播未按 `device_id` 过滤，学生端 `command_handler.rs` 增加 ID 比对逻辑
- **设备注册状态始终 pending**：注册时设为 `pending` 但无审核流程，改为直接 `verified`
- **图片列表 400 错误**：`inspection_id` 原为必填参数但前端未传，改为可选参数，不传时返回最近 50 张
- **教师端独立前端服务**：原需额外启动 `vite` 开发服务器，现教师端内嵌 `static/` 目录并通过 SPA fallback 一体化部署（端口 8080）

## 八、P8 计划：部署与安全加固

### P8 任务清单

| 任务 | 优先级 | 说明 |
|------|--------|------|
| 教师端 Windows Service 注册 | 高 | 使用 `sc create` 或 `nssm` 将 teacher-server 注册为系统服务，开机自启 |
| 学生端开机自启 | 高 | 注册 `agent-core.exe` + `campus-guard.exe` 为计划任务或服务 |
| 安装器打包（NSIS/Inno Setup） | 高 | 一键安装脚本，含 MSVC 运行时检测、防火墙放行、目录结构创建 |
| 防火墙自动放行 | 中 | 安装器自动添加 Windows 防火墙入站规则（8080 端口） |
| SQLite 数据库备份与恢复策略 | 中 | 定期备份、首次启动数据目录初始化 |
| 日志轮转与清理 | 中 | 教师端/学生端日志按天轮转、过期自动清理 |
| API Token 认证 | 中 | 设备注册/API 调用使用预置 Token 验签 |
| HTTPS 支持 | 低 | 可选自签名证书或 let's encrypt |
| 生产部署说明文档 | 高 | 教室实际部署拓扑、网络要求、启动顺序、故障排查 |

### P6-P7 联调验证

- [x] Windows 端到端签到流程验证
- [ ] Windows 端到端检查报告验证
- [ ] Windows 端到端告警处理与图片上传验证
- [x] Windows 端到端硬件快照上报验证
- [x] Windows 端到端进程守护与锁屏验证（模式切换→campus-lock spawn→密码解锁→教师端状态同步）
- [x] 仪表盘统计准确性验证
- [x] 模式切换端到端验证（含锁屏联动）
- [ ] 签到记录导出（CSV/Excel）
- [ ] 图片压缩功能
- [x] 教师端实时感知学生端解锁状态

## 九、开发规范执行情况

- 每次阶段切换先读取 `/workspace/.monkeycode/MEMORY.md`。
- 提交代码前等待用户明确授权。
- Git 提交身份使用 `huchenyang <hcy_1987@163.com>`。
- 每一步开发前生成执行基线文档。
- 本轮联调完成后已同步更新开发总结和 Windows 发布文档。

---

报告更新时间：2026-06-21  
报告版本：v1.5  
当前状态：P0-P7 开发与联调修复完成（含锁屏全链路 + campus-lock IME 修复），P8 部署与安全加固待开始
