# CampusLink-NCMS

CampusLink-NCMS 是一个网络教室使用管理系统，包含教师端服务、教师端 Web 管理界面、学生端原生 Agent、实时通信协议与部署资源。

## 当前状态

- P0-P11 已完成：注册心跳、模式切换、签到检查、硬件日志、部署安全、120 并发压测
- P12 已推送：班级座位、强制签到准入、考勤看板、设备使用记录（`d85c0f7`）
- 启动合并与锁屏恢复已落地，待提交：双击 `teacher.exe` / `student.exe`；解锁恢复锁定前模式
- 当前发布包：`dist/windows-release/`，说明见 `dist/windows-release/README.md`

阶段性成果见 `docs/p12-stage-summary.md`；P0-P11 历史见 `docs/development-summary.md`

## 文档导航

- 文档总索引：`docs/README.md`

### 开发起点

1. `docs/p12-stage-summary.md`
2. `docs/README.md`
3. `docs/development-summary.md`
4. `docs/requirements-analysis.md`

### P0 基线文档

- `docs/p0-execution-plan.md`
- `docs/p0-database-migration-execution-plan.md`
- `docs/p0-proto-files-execution-plan.md`

### P0 可编码子文档

- `docs/p0-database-migration-plan.md`
- `docs/p0-protocol-field-spec.md`
- `docs/p0-backend-api-contract.md`
- `docs/p0-frontend-page-tasks.md`

## 开发规范

- 每一步开发开始前先生成一份执行基线文档
- 执行基线文档格式参照 `docs/p0-execution-plan.md`
- 后续阶段建议命名为 `docs/p1-execution-plan.md`、`docs/p2-execution-plan.md` 等
- 编码、联调、验收都以当前阶段执行基线文档为准

## 当前推荐动作

1. 读 `docs/p12-stage-summary.md` 了解本阶段成果与真机替换要点
2. 教师机双击 `teacher.exe`，学生机双击 `student.exe`，需要守护再开 `campus-guard.exe`
3. 启动合并与锁屏恢复代码待用户授权后提交

## 模块划分

- `teacher-server/`：Rust + Axum 教师端服务
- `teacher-web/`：Vue3 + TypeScript 教师端管理界面
- `student-agent/`：Rust 学生端主进程、守护、锁屏、强制签到
- `proto/`：WebSocket + Protobuf 通信协议定义
- `deploy/`：Windows 与 Linux 部署资源
- `docs/`：需求、架构、数据库、开发计划文档

## 核心能力

- 学生机注册、心跳、UDP 发现教师端
- 班级、座位、学生信息管理
- 开放 / 授课 / 考试 / 锁定模式热切换
- 强制签到准入、环境设备检查、考勤看板
- 设备使用记录、远程锁屏与解锁恢复
- 硬件健康、进程守护、维修工单、日志审计
- 网络认证与超级密码应急解锁
