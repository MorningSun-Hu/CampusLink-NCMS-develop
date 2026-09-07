# P5 执行基线文档：锁屏功能深化与 WebSocket 心跳完善

## 一、阶段目标

在 P0-P4 打通最小业务闭环的基础上，深化学生端 Agent 能力，实现：
1. 完整的 WebSocket 心跳循环与指令接收
2. 实际的锁屏控制能力（`campus-lock`）
3. 模式切换的实际生效逻辑

## 二、功能范围

### 5.1 学生端 WebSocket 心跳循环

**功能描述：**
- 学生端 Agent 主动连接教师端 WebSocket 服务
- 定期上报心跳（当前模式心跳间隔待定义，建议 30 秒）
- 接收教师端下发的模式切换指令
- 接收教师端下发的其他控制指令（如锁屏、解锁、重启等）
- 断线自动重连（退避策略）

**验收标准：**
- [ ] 学生端启动后自动连接 WebSocket
- [ ] 心跳消息按固定间隔上报
- [ ] 教师端可查看设备在线状态
- [ ] 模式切换指令可实时推送到学生端
- [ ] 网络断开后自动重连（最多 3 次，间隔递增）
- [ ] 学生端日志记录连接状态变化

### 5.2 campus-lock 锁屏功能

**功能描述：**
- 实现跨平台锁屏能力（Windows/Linux）
- 支持全屏遮罩锁定
- 支持快捷键拦截（Ctrl+Alt+Del、Alt+Tab、Win 键等）
- 支持解锁密码验证
- 支持远程解锁指令

**验收标准：**
- [ ] Windows 平台可实现屏幕遮罩
- [ ] 快捷键拦截生效（学生无法切换到其他应用）
- [ ] 解锁密码验证正确
- [ ] 远程解锁指令可触发解锁
- [ ] 锁屏状态记录到日志

### 5.3 模式实际生效逻辑

**功能描述：**
- 学生端接收模式切换指令后，执行对应的锁定策略
- 不同模式对应不同的锁定强度：
  - **开放模式**：解除所有限制
  - **授课模式**：锁定屏幕，学生跟随教师演示
  - **考试模式**：全屏锁定，禁止切换应用
  - **锁定模式**：完全锁定，仅保留解锁入口（仅可由教师端远程解锁或超级密码解锁）

> 注：早期设计中的"条件开放模式"（conditional_open）已从产品设计中移除，
> 不再作为可选课堂模式。系统保留四种模式：开放 / 授课 / 考试 / 锁定。

**验收标准：**
- [ ] 每种模式都有对应的锁定行为
- [ ] 模式切换响应时间 < 2 秒
- [ ] 模式切换结果上报教师端
- [ ] 操作日志记录模式变更

## 三、技术实现方案

### 3.1 学生端 WebSocket 心跳循环

**架构设计：**
```
┌─────────────────┐
│  agent-core     │
│  ┌───────────┐  │
│  │ WebSocket │  │
│  │  Client   │  │
│  └─────┬─────┘  │
│        │        │
│  ┌─────▼─────┐  │
│  │   Heart   │  │
│  │   Beat    │  │
│  └─────┬─────┘  │
│        │        │
│  ┌─────▼─────┐  │
│  │  Command  │  │
│  │  Handler  │  │
│  └───────────┘  │
└─────────────────┘
```

**关键实现：**
1. **连接管理**
   - 使用 `tokio-tungstenite` 库建立 WebSocket 连接
   - 连接参数：`ws://<teacher_host>:<port>/ws?device_id=<id>&token=<token>`
   - 连接成功后进入消息循环

2. **心跳上报**
   - 使用 `tokio::time::interval` 定时发送心跳
   - 心跳消息格式：`HeartbeatRequest { device_id, timestamp, status }`
   - 心跳间隔：30 秒（可配置）

3. **指令接收**
   - 异步监听 WebSocket 消息
   - 解析 Protobuf 消息
   - 根据消息类型分发到对应处理器

4. **断线重连**
   - 检测到连接断开后启动重连
   - 使用指数退避：1s, 2s, 4s, 8s, 16s（最多 5 次）
   - 重连成功后继续心跳循环

**代码结构：**
```rust
// student-agent/agent-core/src/websocket.rs
pub struct WebSocketClient {
    device_id: String,
    teacher_fingerprint: String,
    config: Config,
}

impl WebSocketClient {
    pub async fn connect(&mut self) -> Result<()>;
    pub async fn send_heartbeat(&mut self) -> Result<()>;
    pub async fn run(&mut self) -> Result<()>;  // 消息循环
}

// student-agent/agent-core/src/heartbeat_loop.rs
pub struct HeartbeatLoop {
    interval: Duration,
    ws_client: WebSocketClient,
}

impl HeartbeatLoop {
    pub async fn start(&mut self) -> Result<()>;
}

// student-agent/agent-core/src/command_handler.rs
pub async fn handle_mode_switch(cmd: ModeSwitchCommand) -> Result<()>;
pub async fn handle_lock_screen(cmd: LockScreenCommand) -> Result<()>;
pub async fn handle_unlock(cmd: UnlockCommand) -> Result<()>;
```

### 3.2 campus-lock 锁屏实现

**架构设计：**
```
┌─────────────────┐
│  campus-lock    │
│  ┌───────────┐  │
│  │   Lock    │  │
│  │  Screen   │  │
│  └─────┬─────┘  │
│        │        │
│  ┌─────▼─────┐  │
│  │   Input   │  │
│  │  Blocker  │  │
│  └─────┬─────┘  │
│        │        │
│  ┌─────▼─────┐  │
│  │  Password │  │
│  │   Dialog  │  │
│  └───────────┘  │
└─────────────────┘
```

**关键实现：**
1. **Windows 平台**
   - 使用 Windows API `BlockInput` 拦截输入
   - 创建全屏无边框窗口作为遮罩
   - 使用 `SetWindowsHookEx` 拦截键盘消息
   - 解锁对话框使用 `DialogBox` 创建

2. **Linux 平台**
   - 使用 X11/Wayland API 创建全屏窗口
   - 使用 `xinput` 禁用输入设备（可选）
   - 使用系统屏保机制（可选）

3. **密码验证**
   - 解锁密码由教师端下发（临时密码）
   - 或使用学生账号密码（从 config 读取）
   - 验证请求发送到教师端 API

**代码结构：**
```rust
// student-agent/campus-lock/src/lib.rs
pub struct LockManager {
    screen: ScreenOverlay,
    input_blocker: InputBlocker,
}

impl LockManager {
    pub fn lock(&mut self, mode: ModeType) -> Result<()>;
    pub fn unlock(&mut self) -> Result<()>;
    pub fn verify_password(&self, password: &str) -> Result<bool>;
}

// student-agent/campus-lock/src/screen_overlay.rs
pub struct ScreenOverlay {
    window: Window,
}

impl ScreenOverlay {
    pub fn show(&mut self) -> Result<()>;
    pub fn hide(&mut self) -> Result<()>;
}

// student-agent/campus-lock/src/input_blocker.rs
pub struct InputBlocker;

impl InputBlocker {
    pub fn block(&self) -> Result<()>;
    pub fn unblock(&self) -> Result<()>;
}
```

### 3.3 模式实际生效逻辑

**模式与锁定策略映射：**
| 模式 | 锁定强度 | 行为 |
|------|----------|------|
| 开放模式 | 无 | 解除所有限制，恢复正常桌面 |
| 授课模式 | 中 | 锁定学生屏幕，跟随教师演示（待实现投屏接收） |
| 考试模式 | 高 | 全屏锁定，禁止切换应用，禁用 USB |
| 锁定模式 | 最高 | 完全锁定，仅保留解锁入口（教师端远程解锁 / 超级密码解锁） |

**实现方案：**
```rust
// student-agent/agent-core/src/mode.rs
pub enum ModeType {
    Open = 0,           // 开放模式
    Teaching = 1,       // 授课模式
    // 2 预留（原"条件开放模式"已从产品移除，保留编号以兼容存量协议）
    Exam = 3,           // 考试模式
    Locked = 4,         // 锁定模式
}

pub struct ModeManager {
    current_mode: ModeType,
    lock_manager: LockManager,
}

impl ModeManager {
    pub async fn switch_mode(&mut self, mode: ModeType) -> Result<()> {
        match mode {
            ModeType::Open => self.disable_all_locks().await?,
            ModeType::Teaching => self.enable_teaching_mode().await?,
            ModeType::Exam => self.enable_exam_mode().await?,
            ModeType::Locked => self.lock_completely().await?,
        }
        self.current_mode = mode;
        self.report_mode_change(mode).await?;
        Ok(())
    }
}
```

## 四、任务拆解

### 任务 1：学生端 WebSocket 心跳循环

**文件清单：**
- `student-agent/agent-core/src/websocket.rs`（新建）
- `student-agent/agent-core/src/heartbeat_loop.rs`（新建）
- `student-agent/agent-core/src/command_handler.rs`（新建）
- `student-agent/agent-core/Cargo.toml`（更新依赖）

**依赖添加：**
```toml
[dependencies]
tokio-tungstenite = "0.21"
tokio-native-tls = "0.3"  # 如需 wss 支持
```

**实现步骤：**
1. 创建 `WebSocketClient` 结构体，实现连接逻辑
2. 实现心跳定时发送
3. 实现消息循环，解析 Protobuf 消息
4. 实现断线重连逻辑
5. 集成到 `main.rs` 启动流程

### 任务 2：campus-lock 锁屏功能

**文件清单：**
- `student-agent/campus-lock/src/lib.rs`（重写）
- `student-agent/campus-lock/src/screen_overlay.rs`（新建）
- `student-agent/campus-lock/src/input_blocker.rs`（新建）
- `student-agent/campus-lock/src/password_dialog.rs`（新建）
- `student-agent/campus-lock/Cargo.toml`（更新依赖）

**依赖添加（Windows）：**
```toml
[target.'cfg(windows)'.dependencies]
windows = { version = "0.52", features = [
    "Win32_Foundation",
    "Win32_UI_WindowsAndMessaging",
    "Win32_UI_Input_KeyboardAndMouse",
]}

[target.'cfg(windows)'.dependencies]
winit = "0.29"
```

**依赖添加（Linux）：**
```toml
[target.'cfg(target_os = "linux")'.dependencies]
x11 = "2.21"
winit = "0.29"
```

**实现步骤：**
1. 实现全屏遮罩窗口（跨平台）
2. 实现输入拦截（Windows 优先，Linux 后续）
3. 实现解锁对话框
4. 实现密码验证 API 调用
5. 测试锁屏/解锁流程

### 任务 3：模式实际生效逻辑

**文件清单：**
- `student-agent/agent-core/src/mode.rs`（重写，已有占位实现）
- `student-agent/agent-core/src/mode_manager.rs`（新建）
- `student-agent/agent-core/src/main.rs`（更新）

**实现步骤：**
1. 定义模式枚举和锁定策略映射
2. 实现 `ModeManager`，集成 `LockManager`
3. 实现各模式的锁定行为
4. 实现模式变更上报
5. 集成到命令处理器

### 任务 4：联调验证

**验证步骤：**
1. 启动教师端服务和 Web
2. 启动学生端 Agent
3. 验证 WebSocket 连接和心跳
4. 验证模式切换实际生效
5. 验证锁屏/解锁功能
6. 验证断线重连

## 五、验收标准

### 功能验收

- [ ] 学生端启动后自动连接 WebSocket
- [ ] 心跳按 30 秒间隔上报
- [ ] 教师端可查看设备在线状态
- [ ] 模式切换指令实时推送到学生端
- [ ] 学生端执行对应锁定行为
- [ ] 锁屏功能在 Windows 平台可用
- [ ] 解锁密码验证生效
- [ ] 远程解锁指令生效
- [ ] 断线后自动重连（最多 5 次）

### 性能验收

- [ ] 模式切换响应时间 < 2 秒
- [ ] 心跳上报不阻塞主线程
- [ ] 锁屏/解锁无卡顿

### 代码质量验收

- [ ] 所有新增代码通过 `cargo clippy`
- [ ] 所有测试通过 `cargo test`
- [ ] 关键函数有单元测试
- [ ] 错误处理完整（无 `unwrap()` 滥用）
- [ ] 日志记录清晰

## 六、风险与应对

### 风险 1：跨平台锁屏实现难度大

**应对：**
- P5 阶段优先实现 Windows 平台
- Linux 平台先实现基础遮罩，输入拦截后续完善
- 明确告知用户平台支持情况

### 风险 2：快捷键拦截可能失效

**应对：**
- 使用系统级 Hook（Windows `SetWindowsHookEx`）
- 多方式组合拦截（Hook + 系统策略）
- 允许部分系统快捷键（如 Ctrl+Alt+Del）作为应急出口

### 风险 3：WebSocket 重连风暴

**应对：**
- 实现指数退避算法
- 添加随机抖动（jitter）
- 限制最大重连次数

## 七、下一步

1. 本执行基线文档经用户确认后生效
2. 按任务 1→2→3→4 的顺序依次实现
3. 每个任务完成后进行单元验证
4. 所有任务完成后进行端到端联调
5. 生成 P5 联调验证报告并提交

---

**文档版本：** v1.0  
**生成时间：** 2026-06-08  
**状态：** 待用户确认
