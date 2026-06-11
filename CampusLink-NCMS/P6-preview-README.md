# CampusLink-NCMS P6 阶段开发预览

## 版本信息

- **版本**: v0.6.0-preview
- **构建日期**: 2026-06-08
- **阶段**: P6 开发中（签到与检查流程）
- **平台**: Linux x86_64（Windows 需交叉编译）

## 当前开发现状

### 已完成（P0-P5）

| 模块 | 状态 | 文件位置 |
|------|------|---------|
| 教师端服务 | ✅ P5 | teacher-server/ |
| 教师端 Web | ✅ P5 | teacher-web/ |
| 学生端 agent-core | ✅ P5 | student-agent/agent-core/ |
| 学生端 campus-guard | ✅ P5 | student-agent/campus-guard/ |
| 学生端 campus-lock | ✅ P5 框架 | student-agent/campus-lock/ |
| WebSocket 心跳 | ✅ P5 | student-agent/agent-core/src/websocket.rs |
| 模式管理 | ✅ P5 | student-agent/agent-core/src/mode.rs |

### P6 开发进度

| 模块 | 状态 | 说明 |
|------|------|------|
| 数据库迁移 | ✅ 已完成 | migrations/0006_*, 0007_*.sql |
| 签到 API | ✅ 已编写 | src/api/attendance.rs |
| 告警 API | ✅ 已编写 | src/api/alerts.rs |
| 图片上传 API | ✅ 已编写 | src/api/photos.rs |
| 教师端编译 | ⚠️ 待修复 | 编译错误修复中 |
| 前端页面 | ⬜ 待开发 | 签到/告警页面 |

## 编译产物（当前平台 linux）

### 学生端

```bash
student-agent/target/release/
├─ agent-core       (7.4MB) - 主进程
├─ campus-guard     (2.0MB) - 守护进程
└─ campus-lock      (2.0MB) - 锁屏进程（框架）
```

### 测试运行

```bash
cd student-agent
cargo run --package agent-core
```

## 跨平台编译到 Windows

### 方法 1: 使用 cargo-xwin

```bash
cargo install cargo-xwin
cargo xwin build --release --target x86_64-pc-windows-msvc
```

### 方法 2: 使用 Windows 环境

1. 在 Windows 机器上安装 Rust
2. `cd student-agent && cargo build --release`
3. 生成 `.exe` 文件

## P6 API 清单

### 签到管理

- `POST /api/attendance/check-in` - 签到
- `POST /api/attendance/check-out` - 签退
- `POST /api/attendance/retroactive` - 补签
- `GET /api/attendance/statistics` - 统计查询

### 检查与告警

- `POST /api/alerts/submit` - 提交检查/报告异常
- `GET /api/alerts/list` - 告警列表
- `GET /api/alerts/:id` - 告警详情
- `POST /api/alerts/:id/resolve` - 处理告警

### 图片上传

- `POST /api/photos/upload` - 上传图片
- `GET /api/photos/:file_name` - 获取图片
- `POST /api/photos/associate/:photo_id/:alert_id` - 关联照片到告警

## 下一步计划

1. **修复教师端编译错误**
   - 解决 Axum 路由冲突
   - 验证 API 功能

2. **前端开发**
   - 签到管理页面
   - 告警处理页面（含图片上传）

3. **端到端联调**
   - 学生端签到流程
   - 异常报告 → 告警 → 图片上传

4. **完善 campus-lock**
   - Windows 全屏遮罩
   - 输入拦截

## 技术栈

- **教师端**: Rust + Axum + SQLx + SQLite
- **学生端**: Rust + Tokio + WebSocket
- **前端**: Vue 3 + TypeScript + Element Plus
