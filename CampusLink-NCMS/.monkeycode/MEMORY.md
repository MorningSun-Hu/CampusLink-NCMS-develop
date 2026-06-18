# 用户指令记忆

本文件记录了用户的指令、偏好和教导，用于在未来的交互中提供参考。

## 格式

### 用户指令条目
用户指令条目应遵循以下格式：

[用户指令摘要]
- Date: [YYYY-MM-DD]
- Context: [提及的场景或时间]
- Instructions:
  - [用户教导或指示的内容，逐行描述]

### 项目知识条目
Agent 在任务执行过程中发现的条目应遵循以下格式：

[项目知识摘要]
- Date: [YYYY-MM-DD]
- Context: Agent 在执行 [具体任务描述] 时发现
- Category: [运维部署|构建方法|测试方法|排错调试|工作流协作|环境配置]
- Instructions:
  - [具体的知识点，逐行描述]

## 去重策略
- 添加新条目前，检查是否存在相似或相同的指令
- 若发现重复，跳过新条目或与已有条目合并
- 合并时，更新上下文或日期信息
- 这有助于避免冗余条目，保持记忆文件整洁

## 条目

### 教师端编译与运行
- Date: 2026-06-18
- Context: Agent 在编译 P7 变更时发现
- Category: 构建方法
- Instructions:
  - Linux 构建：`cargo build --release`（工作目录 `teacher-server/`）
  - Windows 交叉编译：`cargo build --release --target x86_64-pc-windows-gnu`（工作目录 `teacher-server/` 或 `student-agent/`）
  - 教师端启动前需在 `teacher-server/` 目录下执行 `./target/release/teacher-server`（需要 `config/config.toml`）
  - 前端构建：`npx vite build`（工作目录 `teacher-web/`）
  - 前端开发服务器：`npx vite --host 0.0.0.0`（工作目录 `teacher-web/`）

### Windows 发布包结构
- Date: 2026-06-18
- Context: Agent 在执行 P7 发布包更新时发现
- Category: 运维部署
- Instructions:
  - 发布包目录：`dist/windows-release/`
  - 教师端 exe 需复制到两个位置：`dist/windows-release/teacher-server.exe` 和 `dist/windows-release/teacher-server/teacher-server.exe`
  - 学生端 exe 部署到 `dist/windows-release/student-agent/`（agent-core.exe、campus-guard.exe、campus-lock.exe）
  - 交叉编译目标：`x86_64-pc-windows-gnu`

### 前端路由与侧边栏
- Date: 2026-06-18
- Context: Agent 在添加 P7 页面时发现
- Category: 构建方法
- Instructions:
  - 新增页面后需同时更新 `src/router/index.ts`（子路由）和 `src/components/Layout.vue`（侧边栏菜单项）
  - 侧边栏使用 `router-link` + `custom` v-slot 而非 `el-menu` router 模式，后者存在竞态导致导航失效

### 签到模块数据对齐
- Date: 2026-06-18
- Context: Agent 在修复签到页导航 bug 时发现
- Category: 排错调试
- Instructions:
  - 后端 `/api/attendance/statistics` 返回对象（含 `records` 数组），非数组
  - 后端 `AttendanceRecord` 字段为 snake_case：`check_in_time`、`check_out_time`、`status`
  - 前端 `el-table :data` 只能接受数组，接受对象会导致渲染错误阻塞 Vue Router 导航
  - 补签参数：`student_id`（必填）、`device_id`、`check_in_time`（"YYYY-MM-DD HH:mm:ss"）、`remarks`
