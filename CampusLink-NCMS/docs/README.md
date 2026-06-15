# 文档索引

## 一、基础文档

- `requirements-analysis.md`：功能需求、角色边界、非功能要求
- `architecture-overview.md`：系统分层与通信关系
- `project-structure.md`：项目目录结构设计
- `database-outline.md`：数据库核心表概要

## 二、开发计划文档

- `development-task-breakdown.md`：全量任务拆解、里程碑、优先级、推荐顺序
- `module-roadmap.md`：阶段路线、优先级和开发顺序总览
- `development-summary.md`：当前开发总结报告（最新）

## 三、P0-P6 执行文档

### 执行基线

- `p0-execution-plan.md`：P0 总执行基线
- `p1-device-registration-execution-plan.md`：P1 设备注册执行基线
- `p2-teacher-web-execution-plan.md`：P2 教师端 Web 执行基线
- `p3-student-agent-execution-plan.md`：P3 学生端 Agent 执行基线
- `p4-mode-switch-execution-plan.md`：P4 模式切换执行基线
- `p5-execution-plan.md`：P5 WebSocket 心跳与锁屏框架执行基线
- `p6-execution-plan.md`：P6 签到与检查流程执行基线

### 落地文档

- `p0-database-migration-plan.md`：P0 数据库迁移清单
- `p0-database-migration-execution-plan.md`：P0 数据库迁移文件执行基线
- `p0-protocol-field-spec.md`：P0 协议字段定义
- `p0-proto-files-execution-plan.md`：P0 `.proto` 文件执行基线
- `p0-backend-api-contract.md`：P0 后端接口契约
- `p0-frontend-page-tasks.md`：P0 前端页面任务单
- `p4-validation-report.md`：P4 联调验证报告

## 四、发布与联调文档

- `../WINDOWS-BUILD-SUMMARY.md`：Windows 发布包构建、联调修复与验证结果
- `../dist/windows-release/README-windows.md`：Windows 发布包使用说明

## 五、使用顺序

1. 先读 `requirements-analysis.md`
2. 再读 `development-summary.md`（了解当前进度）
3. 再读 `development-task-breakdown.md`
4. 开始新阶段开发时读对应执行基线文档
