# P7 执行基线文档：硬件快照与审计能力

## 当前前置状态（2026-06-15）

P0-P6 开发已完成，P6 分支 `cb82479` 已提交。教师端 API + 学生端模块 + 前端页面均已实现。P7 可基于已验证的设备注册、心跳链路、WebSocket 通道以及 P6 新增的检查/签到能力继续推进。

本阶段前置约束：
- 教师端继续使用 `sqlx::migrate!` 自动迁移
- P7 新增表结构新增独立 migration 文件
- Windows 发布包更新后同步覆盖 `dist/windows-release/`
- 学生端 `agent-core` 扩展后在 `main.rs` 集成新模块

## 一、阶段目标

在 P6 完善的签到检查流程基础上，实现硬件快照采集与变更告警、日志中心、自定义进程守护，并完善 campus-lock 锁屏能力。

## 二、功能范围

### 7.1 硬件快照采集与变更告警

**功能描述：**
- 学生端启动时采集本机硬件信息（CPU、内存、磁盘、网卡、GPU）
- 定期（如每次心跳或每日）采集并上报教师端
- 教师端对比历史快照，检测硬件变更（新增/移除/替换）
- 硬件变更触发告警，记录变更详情

**验收标准：**
- [ ] 学生端可采集完整硬件信息
- [ ] 硬件信息上报教师端存储
- [ ] 教师端可查询设备硬件快照
- [ ] 硬件变更自动检测并告警
- [ ] 教师端 Web 可查看硬件快照

### 7.2 日志中心

**功能描述：**
- 教师端统一存储各类操作日志
- 支持按类型（注册/签到/检查/锁定/告警）、时间、设备筛选
- 日志导出（CSV）
- 前端日志查询页面

**验收标准：**
- [ ] 教师端 API 支持日志查询与筛选
- [ ] 教师端 Web 日志查询页面
- [ ] 支持分页
- [ ] 日志可导出为 CSV
- [ ] 日志按类型分类展示

### 7.3 自定义进程守护

**功能描述：**
- 教师端下发进程守护策略（目标进程名、守护周期）
- 学生端定期扫描目标进程，发现未运行时自动拉起
- 异常阈值（如连续 N 次拉起失败）触发告警

**验收标准：**
- [ ] 教师端可配置进程守护策略
- [ ] 策略通过 WebSocket 下发给学生端
- [ ] 学生端按策略扫描并守护目标进程
- [ ] 拉起失败触发告警上报

### 7.4 campus-lock 锁屏完善

**功能描述：**
- 完善锁屏进程：全屏遮罩覆盖、键盘输入拦截
- 超级密码解锁流程
- 防强制关闭机制（守护进程监控锁屏进程状态）

**验收标准：**
- [ ] campus-lock 启动后全屏遮罩
- [ ] 拦截 Alt+Tab、Win 键等快捷键
- [ ] 超级密码可解锁
- [ ] 锁屏进程被强行关闭时 campus-guard 自动拉起
- [ ] 解锁操作记录审计日志

## 三、技术实现方案

### 3.1 数据库设计

```sql
-- 硬件快照表
CREATE TABLE hardware_snapshots (
    id TEXT PRIMARY KEY,
    device_id TEXT NOT NULL,
    cpu_model TEXT,
    cpu_cores INTEGER,
    total_memory_bytes INTEGER,
    disk_info TEXT,  -- JSON: [{name, total_bytes, free_bytes}]
    mac_addresses TEXT,  -- JSON: [mac1, mac2]
    gpu_info TEXT,  -- JSON: [{name, driver_version}]
    os_version TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (device_id) REFERENCES student_devices(id)
);

-- 硬件变更记录表
CREATE TABLE hardware_changes (
    id TEXT PRIMARY KEY,
    device_id TEXT NOT NULL,
    change_type TEXT NOT NULL,  -- added, removed, modified
    field_name TEXT NOT NULL,
    old_value TEXT,
    new_value TEXT,
    detected_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (device_id) REFERENCES student_devices(id)
);

-- 进程守护策略表
CREATE TABLE process_guard_policies (
    id TEXT PRIMARY KEY,
    device_id TEXT,
    process_name TEXT NOT NULL,
    check_interval_seconds INTEGER DEFAULT 60,
    max_restart_attempts INTEGER DEFAULT 3,
    enabled INTEGER DEFAULT 1,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (device_id) REFERENCES student_devices(id)
);
```

### 3.2 教师端 API 设计

**硬件快照 API：**
```
POST /api/hardware/snapshot    提交硬件快照
GET  /api/hardware/:device_id  查询设备硬件快照
GET  /api/hardware/changes     查询硬件变更记录
```

**日志 API：**
```
GET /api/logs                   日志查询（支持 type/device_id/date 筛选，分页）
GET /api/logs/export            导出日志 CSV
```

**进程守护 API：**
```
POST /api/policies/process-guard   创建/更新守护策略
GET  /api/policies/process-guard   查询守护策略列表
DELETE /api/policies/process-guard/:id  删除策略
```

### 3.3 学生端实现

**硬件采集模块：**
```rust
// agent-core/src/hardware.rs
pub struct HardwareSnapshot {
    pub cpu_model: String,
    pub cpu_cores: u32,
    pub total_memory_bytes: u64,
    pub disks: Vec<DiskInfo>,
    pub mac_addresses: Vec<String>,
    pub gpus: Vec<GpuInfo>,
    pub os_version: String,
}

impl HardwareCollector {
    pub fn collect() -> Result<HardwareSnapshot>;
    pub async fn submit(&self, config: &Config) -> Result<()>;
}
```

**进程守护模块：**
```rust
// agent-core/src/process_guard.rs
pub struct ProcessGuardian {
    policies: Vec<ProcessGuardPolicy>,
}

impl ProcessGuardian {
    pub fn new(config: &Config) -> Self;
    pub async fn sync_policies(&mut self) -> Result<()>;
    pub fn check_and_restart(&self) -> Vec<ProcessAlert>;
}
```

### 3.4 教师端 Web 前端

**新增页面：**
```
src/views/
├─ Hardware.vue      # 硬件快照与变更记录
└─ Logs.vue          # 日志查询与导出

src/api/
├─ hardware.ts       # 硬件快照 API
└─ logs.ts           # 日志 API
```

## 四、任务拆解

### 任务 1：数据库迁移

- `teacher-server/migrations/0009_create_hardware_snapshots.sql`
- `teacher-server/migrations/0010_create_hardware_changes.sql`
- `teacher-server/migrations/0011_create_process_guard_policies.sql`

### 任务 2：教师端硬件快照 API

- `teacher-server/src/domain/hardware.rs`
- `teacher-server/src/api/hardware_handlers.rs`
- 路由注册

### 任务 3：教师端日志中心 API

- `teacher-server/src/api/log_handlers.rs`
- 日志查询（筛选 + 分页）
- CSV 导出

### 任务 4：教师端进程守护策略 API

- `teacher-server/src/domain/process_guard.rs`
- `teacher-server/src/api/process_guard_handlers.rs`
- 路由注册

### 任务 5：学生端硬件采集模块

- `agent-core/src/hardware.rs`
- 集成到启动流程（注册后采集上报）

### 任务 6：学生端进程守护模块

- `agent-core/src/process_guard.rs`
- WebSocket 接收策略、定时扫描、异常报告

### 任务 7：campus-lock 锁屏完善

- 全屏遮罩实现
- 键盘输入拦截
- 超级密码解锁流程
- 守护进程监控锁屏进程

### 任务 8：campus-guard 守护进程完善

- 监控主进程 + 锁屏进程
- 异常退出自动拉起
- 异常计数与日志上报

### 任务 9：教师端 Web 硬件快照页

- Hardware.vue
- hardware.ts API 封装

### 任务 10：教师端 Web 日志查询页

- Logs.vue
- logs.ts API 封装

### 任务 11：编译验证与发布包更新

- 教师端 + 学生端全量编译
- Windows 交叉编译
- 发布包覆盖

## 五、验收标准

### 功能验收

- [ ] 学生端可采集硬件信息并上报
- [ ] 教师端可查看设备硬件快照
- [ ] 硬件变更自动检测并告警
- [ ] 日志可按类型/设备/时间筛选
- [ ] 日志可导出 CSV
- [ ] 进程守护策略可下发并在学生端生效
- [ ] 进程异常触发告警
- [ ] campus-lock 全屏锁定 + 超级密码解锁
- [ ] 锁屏进程被关后 campus-guard 自动拉起

### 代码质量验收

- [ ] 所有新增代码通过 `cargo build`
- [ ] 前端通过 `vite build`
- [ ] 错误处理完整
- [ ] 日志记录清晰

## 六、风险与应对

### 风险 1：不同平台硬件信息采集差异

**应对：** 使用 `sysinfo` crate 跨平台采集，Windows/Linux 分别适配

### 风险 2：campus-lock 锁屏安全性

**应对：** 全屏遮罩窗口设置 `TOPMOST`，hook 系统快捷键，守护进程定期检查锁屏进程存活

---

**文档版本：** v1.0  
**生成时间：** 2026-06-15  
**状态：** 待用户确认
