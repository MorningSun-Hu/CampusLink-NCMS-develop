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
| P7 | 硬件快照、日志中心、进程守护与锁屏完善 | 完成 | 3 张新表、7 个 API、2 个学生端模块、2 个前端页面、锁屏守护完善 |

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
- `POST /api/devices/register`：设备注册
- `POST /api/devices`：设备列表查询
- `POST /api/devices/:id/mode`：模式切换
- `POST /api/attendance/check-in`：学生签到
- `POST /api/attendance/retroactive`：教师补签
- `GET /api/attendance`：签到记录列表
- `GET /api/attendance/statistics`：签到统计
- `GET /api/inspection`：检查记录列表
- `POST /api/alerts/:id/resolve`：处理告警
- `GET /api/alerts`：告警列表
- `POST /api/photos/upload`：图片上传（multipart）
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
- 硬件信息采集与上报（CPU、内存、磁盘、网卡、OS）
- 进程守护策略同步与进程扫描告警

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

### P7 Bug 修复

- **Layout.vue 导航失效**：`el-menu` 的 `router` 模式与 `default-active` 存在竞态，改用 `router-link` + `custom` v-slot 实现侧边栏
- **Attendance.vue 渲染错误**：后端统计 API 返回对象但 `el-table :data` 期望数组，改为卡片布局展示统计数据
- **签到数据字段对齐**：前端 `AttendanceRecord` 字段名对齐后端 snake_case（`check_in_time`/`check_out_time`/`status`）

## 八、待完善功能

### P6-P7 联调验证（待实机测试）

- [ ] Windows 端到端签到流程验证
- [ ] Windows 端到端检查报告验证
- [ ] Windows 端到端告警处理与图片上传验证
- [ ] Windows 端到端硬件快照上报验证
- [ ] Windows 端到端进程守护与锁屏验证
- [ ] 签到记录导出（CSV/Excel）
- [ ] 图片压缩功能

### 后续阶段

- [ ] P8：部署与安全加固（Windows Service 注册、安装器打包、生产部署说明）

## 九、开发规范执行情况

- 每次阶段切换先读取 `/workspace/.monkeycode/MEMORY.md`。
- 提交代码前等待用户明确授权。
- Git 提交身份使用 `huchenyang <hcy_1987@163.com>`。
- 每一步开发前生成执行基线文档。
- 本轮联调完成后已同步更新开发总结和 Windows 发布文档。

---

报告更新时间：2026-06-18  
报告版本：v1.3  
当前状态：P0-P7 开发完成，P6-P7 待 Windows 实机联调
