# CampusLink-NCMS 自动测试报告

## 一、报告信息

- 报告日期：2026-08-29
- 测试方式：后台自动构建 + 自动单元测试 + API 端到端功能验证测试 + 学生端实机运行验证 + 并发压测
- 验证对象：teacher-server、teacher-web、student-agent（agent-core / campus-guard / campus-lock）、全部 15 张数据库迁移、40+ API 端点
- 报告版本：v1.0

## 二、测试环境

- 平台：Linux x86_64（2 核 / 8GB 内存）
- Rust：1.98.0（stable，本机新装）
- Node.js：v22.22.0 / npm 10.9.4
- 数据库：SQLite（测试用临时库，避免污染项目）
- 依赖补齐：`libssl-dev` / `pkg-config`（openssl-sys）、`protobuf-compiler`（prost-build）

## 三、自动构建结果

| 目标 | 结果 | 说明 |
|------|------|------|
| teacher-server 编译 | 通过（5m39s） | 19 个 dead-code 警告 |
| teacher-server cargo test | 0 passed | 全库无单元测试 |
| student-agent 3 crate 编译 | 通过（21m57s） | 约 30 个警告 |
| student-agent cargo test | 0 passed | 全库无单元测试 |
| 前端 vue-tsc + vite build | 通过 | 修复 3 处缺陷后通过 |
| 数据库迁移 | 15/15 完整 | 0001~0015 全部应用 |

### 构建环境前置依赖

- Rust 工具链未预装，需通过 rustup 安装
- teacher-server 依赖系统库 `libssl-dev`、`pkg-config` 与 `protobuf-compiler`
- 前端 vue-tsc 1.8.x 与 typescript 5.9 不兼容，需升至 2.x
- 前端产物需手动同步到 `teacher-server/static/`，SPA fallback 依赖 `static/index.html`

## 四、功能完成度评估

| 维度 | 文档宣称 | 实测 |
|------|---------|------|
| 教师端 API | 34 端点 | 路由齐全，认证、鉴权、核心接口 200 |
| 教师端 Web | 13 页面 | 全部构建通过 |
| 学生端编译 | 4 crate | 通过 |
| 设备注册 / 心跳 / 在线状态 | 完成 | 实机闭环验证通过 |
| WebSocket 心跳 ACK | 完成 | 实机验证通过 |
| 学生端数据上报（签到/硬件/策略/告警） | 全链路完成 | 全部 401，实际不可用 |
| 120 并发压测 | 120/120 | 实测 8/120（database is locked） |

**核心结论：项目完成度约 70%。** 教师端服务、前端、学生端进程均能构建运行，注册/心跳链路可用；但学生端 Agent 的全部数据上报功能在实机中失败，P6-P7 声称完成的阶段实际未闭环。

## 五、功能缺陷清单

### 严重缺陷

#### 1. 学生端 Agent 上报接口全部 401（架构级）

- 现象：实机运行 agent-core 日志明确显示硬件快照、签到、策略同步、告警上报全部返回 HTTP 401（"请先登录"）。
- 根因：
  - `teacher-server/src/api/mod.rs` 将 `/api/attendance/check-in`、`/api/inspection/submit`、`/api/hardware/snapshot`、`/api/policies/process-guard` 等挂到 JWT 保护路由；
  - 学生端 `agent-core` 各模块的 HTTP 请求不携带任何 Authorization 头（学生端仅有 WebSocket 的 session_key 机制）。
- 影响：签到、检查上报、硬件快照、进程守护策略同步、异常告警全部失效。

#### 2. SQLite 并发写锁，压测不达标

- 现象：120 并发注册仅 8/120 成功，服务端大量 `database is locked`（code 5），单条 INSERT 耗时 8~10 秒。
- 根因：`infrastructure/database.rs` 未设置 `busy_timeout`、未启用 WAL，连接池（10 连接）共享单文件写锁。
- 影响：与文档宣称的 120/120 压测通过不符，实际峰值仅约 10 台并发注册。

### 中等级缺陷

#### 3. 数据库备份路径硬编码

- `infrastructure/backup.rs` 固定写死 `data/campuslink.db`；用 `DATABASE_URL` 指定数据库时备份必然失败（启动日志可见 `Source database not found`）。

### 前端缺陷（本轮已修复）

#### 4. Alerts.vue 重复声明与悬挂代码

- `customUpload` 函数重复声明（196/248 行）且存在悬挂残代码 `return map[type]`，导致 `vite build` 直接失败。已修复。

#### 5. Device 类型未导出

- `api/devices.ts` 未从 `types.ts` re-export `Device`，`Devices.vue` / `Monitor.vue` 导入报 TS2459。已修复。

#### 6. vue-tsc 与 typescript 版本不兼容

- vue-tsc 1.8.27 与 typescript 5.9.3 不兼容，`npm run build` 崩溃。已升级 vue-tsc 至 2.2.12。

### 质量缺陷

#### 7. 测试缺失

- 4 个 Rust crate 共 0 个单元测试，前端 0 测试；压测依赖一次性手工工具，无自动化回归保障。

#### 8. 文档过度乐观

- `v4.1-progress-report.md` 宣称"签到全链路验证通过"，与当前代码实际行为矛盾。

## 六、API 端到端功能验证测试明细

总计 40 项（含补充验证），33 项通过，7 项首轮失败。其中 4 项为测试脚本参数问题（repair 需 camelCase、students 缺必填字段等），以正确参数复测通过；3 项为真实缺陷（即上文缺陷 1 对应的三个上报接口：check-in、inspection/submit、hardware/snapshot，无 token 时 401）。

### 全部通过（33 项）

- 健康检查、JWT 登录、未授权访问保护（401）
- 仪表盘概览、设备列表
- 设备模式切换、远程锁屏、远程解锁
- 签到记录列表、签到统计、补签、签到 CSV 导出
- 检查记录列表、告警列表、检查 CSV 导出
- 图片上传、图片列表
- 硬件快照查询、硬件变更记录
- 日志查询、日志 CSV 导出
- 进程守护策略创建、查询
- 学生列表、学生 CSV 导出
- 锁屏密码查询/修改、定时调度查询
- 维修工单列表、网络认证账号列表、设备白名单列表
- SPA 静态首页

### 首轮失败、参数复测后通过（4 项）

- POST /api/students：脚本缺必填字段 password，补全后 200
- POST /api/repair-orders：脚本字段为 snake_case，接口要求 camelCase，修正后 200
- POST /api/network-accounts：脚本字段名不符，修正后 200
- POST /api/alerts/resolve：无告警数据可处理（前置告警未生成）

### 真实缺陷失败（3 项）

- POST /api/attendance/check-in：无 token 401
- POST /api/inspection/submit：无 token 401
- POST /api/hardware/snapshot：无 token 401

## 七、学生端实机运行验证

在临时目录实机运行 agent-core，验证链路：

| 环节 | 结果 |
|------|------|
| UDP 广播发现教师端 | 通过（发现 0.0.0.0:8080） |
| 设备注册 | 通过 |
| WebSocket 连接 | 通过 |
| 心跳发送与 heartbeat_ack 接收 | 通过 |
| 教师端在线状态更新 | 通过（online） |
| 硬件快照上报 | 失败（401） |
| 自动签到 | 失败（401） |
| 进程守护策略同步 | 失败（401） |
| 异常告警上报 | 失败（401） |

## 八、修复建议（按优先级）

1. 学生端上报接口改为公开路由并复用 device_id 校验，或给学生端签发长期 token（推荐：统一走公开路由 + device_id 校验，与 /api/devices/register 一致）。
2. `database.rs` 增加 `busy_timeout(5000)` 与 `journal_mode(WAL)`，解决并发写锁。
3. `backup.rs` 从 Config 读取数据库路径，而非硬编码。
4. 为核心 domain 逻辑补充单元测试，作为回归保障替代一次性压测脚本。

## 九、缺陷修复记录（v1.1）

### 修复 1：学生端上报接口全部 401（已完成）

按建议方案统一走公开路由 + device_id 校验：

- `api/mod.rs`：将 `POST /api/attendance/check-in`、`POST /api/inspection/submit`、`POST /api/hardware/snapshot` 从 JWT 保护路由移入公开路由；新增学生端策略同步公开端点 `GET /api/policies/process-guard/sync`（教师端原 `GET /api/policies/process-guard` 保持 JWT 保护）。
- 三个上报 handler 增加 `device_exists` 校验（`domain/device.rs` 新增函数），未注册设备返回 401。
- agent-core `process_guard.rs`：策略同步 URL 改为公开端点 `/api/policies/process-guard/sync`。

回归验证（12 项 API 测试全部通过）：

- 已注册设备签到/硬件快照/检查上报返回 code 0；未注册设备上报被拒（401）
- 学生端策略同步经公开端点可用；教师端策略列表仍要求 JWT
- 教师端带 token 的统计/策略查询正常

### 修复 2：SQLite 并发写锁（已完成）

`database.rs` 连接选项增加 `busy_timeout(Duration::from_millis(5000))` 与 `journal_mode(SqliteJournalMode::Wal)`。

回归验证（120 并发压测）：

| 指标 | 修复前 | 修复后 |
|------|--------|--------|
| HTTP 注册成功率 | 8/120（6.7%） | 120/120（100%） |
| 注册耗时 | 单条 INSERT 8~10s | 总耗时 997ms |
| WS 连接成功率 | - | 120/120（100%） |
| 心跳 ACK 率 | - | 1440/1440（100%） |

### 修复 3：数据库备份路径硬编码（已完成）

- `backup.rs`：`start()` 接收 `database_url` 参数，通过 `resolve_db_file_path` 解析实际库文件；内存库跳过备份。
- `api/mod.rs` / `main.rs`：将 `config.database.url` 传入 `create_app` 与 `backup::start`。

回归验证：以 `DATABASE_URL` 指向临时库启动，备份日志正确显示从配置路径创建备份，不再出现 `Source database not found`。

### 修复 4：补充 domain 层单元测试（已完成，并额外修复 3 个潜在缺陷）

为 6 个 domain 模块新增 23 个单元测试（内存 SQLite + 迁移），覆盖：签到/补签/统计、设备注册/白名单、硬件快照变更检测、检查上报/告警、进程守护策略 CRUD、JWT 登录与校验。**23/23 全部通过。**

测试编写过程中额外发现并修复 3 个潜在缺陷：

1. **登录绕过漏洞（严重）**：`domain/auth.rs` 中 `bcrypt::verify` 返回 `Ok(false)`（密码错误）时未检查布尔结果，错误密码也能登录成功。已修复为校验 `verified` 值。
2. **签到统计恒为 0**：`domain/attendance.rs` `get_statistics` 在 date 为空时把字符串 `"date('now')"` 作为参数绑定，SQLite 无法解析导致统计数恒为 0。已重写为 None 时直接在 SQL 中写 `date('now')`。
3. **硬件变更日志写入失败**：`domain/hardware.rs` `detect_changes` 向 `operation_logs` 插入了不存在的 `device_id`/`action`/`detail` 列（实际列名 `operator`/`target_id`/`content`），导致硬件变更上报时日志插入报错。已修正列名。

### 修复后重新验证结果

- teacher-server 编译通过（19 个警告），`cargo test` 23/23 通过
- student-agent agent-core `cargo check` 通过（24 个警告）
- API 回归 12/12 通过（含未注册设备拒绝、教师端接口仍受保护）
- 并发压测 120/120 达标

---

报告版本：v1.1  
报告生成：2026-08-29
