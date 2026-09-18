# CampusLink-NCMS 阶段性成果总结（2026-09）

报告日期：2026-09-18
当前分支：`P7`（跟踪 `origin/P7`）
已推送基线：`d85c0f7` feat: 实现签到准入、考勤看板与设备使用记录
工作区状态：启动合并、签到界面与锁屏恢复相关改动已落地，尚未提交

## 一、阶段定位

P0–P11 已完成教师端服务、教师 Web、学生端 Agent、部署与 120 并发压测，形成可交付的教室管理系统骨架。

本阶段（P12）把教室日常使用闭环补齐：班级座位、授课准入、强制签到、考勤看板、设备使用记录，并把 Windows 发布改为双击 `teacher.exe` / `student.exe` 启动。

## 二、能力全景

| 领域 | 状态 | 说明 |
|------|------|------|
| 工程骨架 P0–P5 | 已完成 | 注册、心跳、模式切换、WebSocket、锁屏框架 |
| 签到检查 P6 | 已完成 | 签到/检查 API、前端页、学生端上报 |
| 硬件日志锁屏 P7 | 已完成 | 硬件快照、日志中心、进程策略、远程锁屏 |
| 部署安全 P8–P10 | 已完成 | Windows 安装、JWT、UDP 发现、WS 加密、工单 |
| 压测补齐 P11 | 已完成 | 120 并发注册/心跳通过 |
| 班级座位与签到规则 | 已推送 `17a887c` | 班级、座位绑定、授课/开放签到规则 |
| 强制准入与考勤使用记录 | 已推送 `d85c0f7` | `campus-checkin`、考勤看板、使用记录 |
| 启动合并与锁屏恢复 | 代码已实现，待提交 | 双击 exe 启动；解锁恢复锁定前模式 |

规格目录：

- `.monkeycode/specs/checkin-mode-enhancement/`
- `.monkeycode/specs/checkin-usage-inspection/`（任务 1–13 完成；标 `*` 的单测子任务按约定跳过）

## 三、本阶段功能成果

### 3.1 班级、座位与签到规则（`17a887c`）

- 新增班级实体：学生/设备可归班，座位号在班级内唯一
- 授课模式必须选择班级；班级无学生时拒绝切换
- 座位匹配的设备进入授课，不匹配的设备进入锁定，教师端展示分组数量
- 开放模式签到填姓名；授课模式用学号或姓名+密码，首次强制改密，初始密码 `123456`
- `open`/`teaching` 要求签到；`exam`/`locked` 不要求签到
- 设备重复注册保持稳定主键，避免后续归班/签到失效

### 3.2 强制准入、考勤看板、使用记录（`d85c0f7`）

- 新增 `campus-checkin`：开放/授课模式下全屏准入，身份确认 → 环境设备检查 → 提交后进入桌面
- 准入异常退出会由主进程重新拉起；教师超级密码可放行
- 授课签到写入 `attendance_records`；开放签到写入 `device_usage_records`，不进入考勤
- 考勤看板按班级与日期给出应到、已签、缺勤、出勤率与缺勤名单
- 签到可携带检查项；异常记告警，仍允许进入桌面
- 教师端新增「设备使用」页：查询、手动结束会话、导出

### 3.3 启动合并与实机修复（工作区未提交）

启动方式：

- 教师机双击 `teacher.exe`：自动切到 exe 目录，创建 `data/` `logs/` `config/`，缺失时写入默认 `config.toml`
- 学生机双击 `student.exe`：自动切到自身目录并创建配置
- 需要守护时再双击 `campus-guard.exe`，只守护 `student.exe`

实机问题修复：

- 签到窗口去掉每帧 `Fullscreen`/`AlwaysOnTop` 与常驻 200ms 重绘，忙时才 100ms 重绘，消除闪烁
- 检查页「进入桌面」「重新检查」收回中间卡片
- `taskkill` 使用 `CREATE_NO_WINDOW` 与空 stdio，避免弹出「没有找到进程」
- `campus-guard` 不再拉起 `campus-lock`，避免解锁后被守护进程重新锁屏
- 锁屏/解锁按 `device_id` 下发；心跳回写 `current_mode`；教师端操作后刷新设备列表

解锁恢复锁定前状态：

- 「锁屏」按钮为 overlay：记住 `mode_before_lock`，解锁回到授课/开放，已签到不重弹签到
- 「切换模式 → 锁定」为完整锁定；解锁同样恢复原模式，开放/授课按需启动准入
- `exam`/`locked` 作为恢复目标时回退为 `open`
- 迁移 `0019_device_mode_before_lock.sql` 为 `student_devices.mode_before_lock`

## 四、数据与接口增量

### 4.1 迁移 0016–0019

| 文件 | 内容 |
|------|------|
| `0016_create_classes.sql` | `classes` 表 |
| `0017_add_class_and_signin_fields.sql` | 学生/设备班级与座位、`password_set`、`pending_checkin`、考勤 `seat_no` |
| `0018_device_usage_records.sql` | `device_usage_records`；考勤关联 `usage_record_id` |
| `0019_device_mode_before_lock.sql` | `student_devices.mode_before_lock`（待提交） |

### 4.2 本阶段关键接口

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/api/attendance/board` | 班级日期考勤看板 |
| GET | `/api/attendance/context` | 学生端签到上下文（公开） |
| GET | `/api/usage` | 使用记录查询 |
| GET | `/api/usage/export` | 使用记录导出 |
| POST | `/api/usage/:id/end` | 教师手动结束会话 |
| POST | `/api/usage/close` | 学生端关闭会话（公开） |
| GET/POST | `/api/classes` | 班级列表与创建 |
| POST | `/api/classes/:id/mode` | 班级批量切模式 |
| POST | `/api/classes/:id/seats` | 批量座位 |
| POST | `/api/auth/student-login` | 学生登录（学号或姓名） |
| POST | `/api/auth/student-set-password` | 学生改密 |
| POST | `/api/devices/:id/lock` | overlay 锁屏，记住原模式 |
| POST | `/api/devices/:id/unlock` | 解锁并恢复原模式 |

锁屏 WebSocket 带 `device_id` 与 `restore_mode`；心跳可回写设备当前模式。

## 五、Windows 发布

发布目录：`dist/windows-release/`
压缩包：`dist/CampusLink-NCMS-windows-v1.1.zip`（约 27MB，`dist/` 已 gitignore）
使用说明：`dist/windows-release/README.md`

```text
windows-release/
├── README.md
├── teacher-server/
│   ├── teacher.exe
│   ├── config/config.toml
│   ├── static/
│   └── data/                  运行时创建
└── student-agent/
    ├── student.exe
    ├── campus-guard.exe
    ├── campus-lock.exe
    └── campus-checkin.exe
```

真机替换要点：

1. 学生机同时替换 `student.exe`、`campus-checkin.exe`、`campus-lock.exe`、`campus-guard.exe`，然后重启守护
2. 教师机替换 `teacher.exe`，首次启动自动跑迁移 `0019`
3. 旧版 `campus-guard.exe` 仍会拉起 `campus-lock`，必须一并替换

默认账号：教师 `admin` / `admin123`；学生初始密码 `123456`；锁屏密码默认 `admin123`。

## 六、教师端页面

| 路由 | 页面 |
|------|------|
| `/login` | 登录 |
| `/dashboard` | 仪表盘 |
| `/devices` | 设备、模式切换、锁屏/解锁 |
| `/classes` | 班级、座位、批量切模式 |
| `/students` | 学生 CRUD、导入导出 |
| `/attendance` | 考勤筛选、看板、缺勤名单、导出 |
| `/usage` | 设备使用记录 |
| `/alerts` | 检查告警 |
| `/hardware` | 硬件快照 |
| `/logs` | 日志中心 |
| `/monitor` | 监控总览 |
| `/repairs` | 维修工单 |
| `/settings` | 超级密码、定时切模式 |
| `/network-accounts` | 网络认证账号 |
| `/device-whitelist` | 设备白名单 |

前端构建后需同步到 `teacher-server/static/`。

## 七、构建与验证备忘

- 教师端：`cargo build --bin teacher` / `cargo test --bin teacher`
- 学生端：`cargo build -p agent-core` 产出 `student`；另编 `campus-checkin`、`campus-lock`、`campus-guard`
- Windows 交叉编译：`CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER=x86_64-w64-mingw32-gcc cargo build --release --target x86_64-pc-windows-gnu`
- `cargo test` 不更新运行二进制，改 `main.rs` 后需再 `cargo build`
- 签到进程：`campus-checkin <server_url> <device_id> <mode> [lock_password]`，退出码 0/1/2/10

## 八、待办

1. 用户授权后提交启动合并与锁屏恢复改动（含迁移 `0019` 与前端 static）
2. 真机替换四件套后验证：签到不闪、检查按钮位置、锁屏解锁恢复原模式、守护不再拉起锁屏
3. 规格中标 `*` 的单元测试仍按约定跳过

## 九、文档阅读顺序

1. 本文件：当前阶段成果
2. `docs/development-summary.md`：P0–P11 历史记录
3. `docs/deployment-guide.md`：生产安装与运维
4. `dist/windows-release/README.md`：当前发布包用法
5. 新功能开发前再读 `docs/requirements-analysis.md` 与对应执行基线
