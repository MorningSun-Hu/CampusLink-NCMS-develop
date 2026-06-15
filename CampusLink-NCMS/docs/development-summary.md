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
| P6 | 签到与检查流程 | 进行中 | 执行基线已生成，功能继续集成 |

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

### 2. 教师端服务

接口与能力：
- `GET /api/health`：健康检查
- `POST /api/devices/register`：设备注册
- `POST /api/devices`：设备列表查询
- `POST /api/devices/:id/mode`：模式切换
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

辅助进程：
- `campus-guard`：守护进程框架
- `campus-lock`：锁屏进程框架

### 4. 教师端 Web

页面：
- `/login`：登录页占位
- `/dashboard`：仪表盘
- `/devices`：设备列表与模式切换弹窗
- `/monitor`：监控总览

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

## 七、待完善功能

### P6 阶段

- [ ] 签到流程：发起、记录、统计、补签、导出
- [ ] 检查流程：卫生检查、设备检查、异常分级
- [ ] 告警处理：告警列表、处理状态、图片上传与预览
- [ ] 学生登录：账号密码验证、登录状态持久化
- [ ] 教师端 Web：签到管理页、检查告警页

### P7 阶段

- [ ] 硬件快照与变更告警
- [ ] 日志中心完善
- [ ] 异常处理与导出能力

### P8 阶段

- [ ] Windows Service 注册
- [ ] 安装器打包
- [ ] 生产部署说明

## 八、开发规范执行情况

- 每次阶段切换先读取 `/workspace/.monkeycode/MEMORY.md`。
- 提交代码前等待用户明确授权。
- Git 提交身份使用 `huchenyang <hcy_1987@163.com>`。
- 每一步开发前生成执行基线文档。
- 本轮联调完成后已同步更新开发总结和 Windows 发布文档。

---

报告更新时间：2026-06-15  
报告版本：v1.1  
当前状态：P0-P5 完成，Windows 教师端与学生端注册和 WebSocket 心跳闭环已验证，P6 进行中
