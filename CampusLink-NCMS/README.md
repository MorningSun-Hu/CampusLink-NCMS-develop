# CampusLink-NCMS

CampusLink-NCMS 是一个网络教室使用管理系统，包含教师端服务、教师端 Web 管理界面、学生端原生 Agent、实时通信协议与部署资源。

## 当前状态

- 已完成需求分析、架构拆分、项目结构设计
- 已完成开发路线、优先级、推荐顺序整理
- 已完成 `P0` 总执行基线与配套子文档
- 已具备进入实际开发阶段的文档准备条件

## 文档导航

- 文档总索引：`docs/README.md`

### 开发起点

1. `docs/requirements-analysis.md`
2. `docs/development-task-breakdown.md`
3. `docs/module-roadmap.md`
4. `docs/stage-summary-and-dev-readiness.md`

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

1. 按 `docs/p0-database-migration-execution-plan.md` 生成迁移文件
2. 按 `docs/p0-proto-files-execution-plan.md` 生成 `.proto` 文件
3. 基于上述结果进入 `teacher-server` 基础工程开发

## 模块划分

- `teacher-server/`：Rust + Axum 教师端服务
- `teacher-web/`：Vue3 + TypeScript 教师端管理界面
- `student-agent/`：Rust 学生端 Agent、守护进程、锁屏进程
- `proto/`：WebSocket + Protobuf 通信协议定义
- `deploy/`：Windows 与 Linux 部署资源
- `docs/`：需求、架构、数据库、开发计划文档

## 核心能力

- 学生机注册与认证
- 学生信息与座位管理
- 模式控制与热切换
- 实时监控与远程操作
- 签到、检查、维修、日志审计
- 硬件健康与进程守护告警
- 网络认证与超级密码应急解锁
