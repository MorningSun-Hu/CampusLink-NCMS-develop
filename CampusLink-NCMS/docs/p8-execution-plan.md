# P8 执行基线文档：部署与安全加固

## 当前前置状态（2026-06-19）

P0-P7 开发与联调修复全部完成。教师端已内嵌前端静态文件（SPA fallback），单一端口 8080 一体化部署。学生端 agent-core 可注册、心跳、签到、硬件快照上报、模式切换响应。Windows 发布包 `dist/windows-release/` 已包含完整可执行文件集。

本阶段前置约束：
- 当前为手动启动模式（`start.bat`），需升级为系统服务/计划任务
- 教师端 8080 端口需 Windows 防火墙放行
- 无安装器，需手动解压和配置
- 无 API 认证机制，任意客户端可注册设备
- SQLite 数据库无自动备份，日志无限增长

## 一、阶段目标

将 CampusLink-NCMS 从开发态升级为可部署的生产级系统，实现一键安装、开机自启、防火墙自动配置、数据备份与日志管理、API 认证、生产部署文档。

## 二、功能范围

### 8.1 Windows Service 注册（教师端）

**功能描述：**
- 将 teacher-server.exe 注册为 Windows 系统服务
- 服务随系统启动自动运行
- 支持服务启动/停止/重启
- 异常退出自动恢复

**验收标准：**
- [ ] teacher-server 可注册为 Windows Service
- [ ] 开机后服务自动启动
- [ ] `sc start/stop/query teacher-server` 可管理服务
- [ ] 进程崩溃后服务管理器自动重启（failure action = restart）
- [ ] 安装器自动完成服务注册

### 8.2 学生端开机自启

**功能描述：**
- agent-core.exe 随 Windows 登录自动启动
- campus-guard.exe 在学生端启动后作为守护进程运行
- 启动时机：用户登录后（非系统服务，需要桌面交互权限用于锁屏）

**验收标准：**
- [ ] 学生端通过计划任务或 Startup 文件夹实现登录自启
- [ ] agent-core 启动后 campus-guard 自动拉起
- [ ] 注销/关机时 agent 正常退出并清理状态
- [ ] 安装器自动设置自启

### 8.3 安装器打包

**功能描述：**
- 使用 NSIS 或 Inno Setup 打包为单一 exe 安装程序
- 安装器检测运行环境：Windows 版本、MSVC 运行时
- 自动创建目录结构，释放可执行文件和配置文件
- 教师端安装选项和内嵌前端
- 学生端安装选项
- 支持自定义安装路径
- 防火墙自动放行

**验收标准：**
- [ ] 生成 `CampusLink-NCMS-Setup.exe` 安装程序
- [ ] 安装时检测 MSVC 运行时，缺失则引导安装
- [ ] 安装程序正确释放文件到目标目录
- [ ] 安装完成后服务/计划任务自动注册
- [ ] 支持静默安装模式（`/S` 参数）
- [ ] 安装日志可追溯

### 8.4 防火墙自动放行

**功能描述：**
- 安装时自动添加 Windows Defender 防火墙入站规则
- 放行 TCP 8080 端口（教师端 Web 管理界面）
- 规则名称可识别，避免重复添加
- 卸载时自动移除规则

**验收标准：**
- [ ] 安装后可通过局域网其他机器访问教师端 http://IP:8080
- [ ] 防火墙规则命名清晰，可在 Windows 防火墙中查看
- [ ] 重复安装不产生重复规则
- [ ] 卸载时自动清理规则

### 8.5 数据库备份与恢复

**功能描述：**
- 教师端启动时自动备份现有 SQLite 数据库（每日一份）
- 备份文件命名规则：`backup/YYYY-MM-DD/campuslink.db`
- 保留最近 7 天备份，自动清理过期
- 支持从备份恢复：将备份文件复制到数据目录即可

**验收标准：**
- [ ] 启动时自动创建 `data/backup/YYYY-MM-DD/` 目录
- [ ] 成功备份原数据库文件
- [ ] 7 天前备份自动删除
- [ ] 恢复流程文档清晰

### 8.6 日志轮转与清理

**功能描述：**
- 教师端日志按天轮转：`logs/teacher-server-YYYY-MM-DD.log`
- 教师端保留最近 14 天日志
- 学生端日志按天轮转：`logs/agent-core-YYYY-MM-DD.log`
- 学生端保留最近 7 天日志
- 启动时自动清理过期日志文件

**验收标准：**
- [ ] 日志文件名含日期，每天生成新文件
- [ ] 过期日志自动删除
- [ ] 日志大小可控，不撑满磁盘
- [ ] 部署后无需手动清理日志

### 8.7 API Token 认证

**功能描述：**
- 教师端首次启动生成随机 Admin Token（写入 `config/config.toml` 和数据库）
- 管理 API（模式切换、设备删除、策略下发）需 Bearer Token 认证
- 设备注册 API 使用预置 Register Token
- 前端页面登录使用 Admin Token
- Token 支持在配置文件中手动更换

**验收标准：**
- [ ] 未认证请求返回 401
- [ ] Bearer Token 正确则放行
- [ ] 首次启动自动生成 Token 并打印到控制台
- [ ] Token 可手动更新
- [ ] WebSocket 连接也需 Token 认证（query param）

### 8.8 HTTPS 支持（可选）

**功能描述：**
- 支持通过配置文件指定 TLS 证书路径
- 提供自签名证书生成脚本
- 默认 HTTP 运行，HTTPS 为可选开启

**验收标准：**
- [ ] 配置 TLS 证书后监听 HTTPS
- [ ] 提供 `generate-cert.bat` 生成自签名证书
- [ ] HTTP 和 HTTPS 不同端口或共存

### 8.9 生产部署说明文档

**功能描述：**
- 教室部署拓扑图（教师机 + N 台学生机）
- 网络要求（教师机固定 IP、同一局域网、防火墙端口）
- 安装步骤（教师端安装 → 学生端安装 → 验证）
- 启动顺序（先教师端后学生端）
- 日常运维（查看状态、备份恢复、Token 管理）
- 故障排查（常见问题与解决方案）

**验收标准：**
- [ ] 部署拓扑图清晰
- [ ] 安装步骤逐条可操作
- [ ] 常见问题覆盖实际场景
- [ ] 文档伴随发布包一起分发

## 三、技术实现方案

### 3.1 教师端 Windows Service

**方案：** 使用 `windows-service` crate 替代手动 `sc create`

```rust
// teacher-server/src/main.rs
#[cfg(windows)]
fn run_as_service() -> Result<()> {
    use windows_service::{
        service::{ServiceControl, ServiceControlAccept, ServiceExitCode, ServiceState, ServiceStatus, ServiceType},
        service_control_handler::{self, ServiceControlHandlerResult},
        service_dispatcher,
    };

    service_dispatcher::start("teacher-server", ffi_service_run)?;
    Ok(())
}
```

**命令行参数：**
```
teacher-server.exe                  # 交互模式运行（开发调试）
teacher-server.exe --install        # 注册为 Windows Service
teacher-server.exe --uninstall      # 卸载 Windows Service
teacher-server.exe --service        # 服务模式运行（由 SCM 调用）
```

**备选方案：** 安装器直接调用 `sc create` + `sc config` 完成服务注册，无需修改 Rust 代码，降低复杂度。

```
sc create "teacher-server" binPath= "C:\CampusLink-NCMS\teacher-server\teacher-server.exe --service" start= auto
sc failure "teacher-server" reset= 86400 actions= restart/60000/restart/60000/restart/60000
sc start "teacher-server"
```

**决策：** 采用备选方案（sc 命令），避免引入 windows-service 依赖及 SCM 生命周期复杂度。teacher-server 本身是长期运行进程，`start.bat` 模式已验证稳定。

### 3.2 学生端开机自启

**方案：** 创建 Windows 计划任务，触发器为用户登录时执行

```bat
schtasks /create /tn "CampusLink Student Agent" /tr "C:\CampusLink-NCMS\student-agent\start.bat" /sc onlogon /rl highest /f
```

### 3.3 安装器脚本

**方案：** 使用 Inno Setup（免费、脚本化、支持 Pascal Script）
- `.iss` 脚本定义安装界面、文件列表、快捷方式
- `[Run]` 段执行 `install.bat` 完成安装后配置
- 支持 64 位检测、MSVC 运行时静默安装

```
install.bat 执行内容：
1. 创建 data/logs/backup 子目录
2. 添加防火墙规则：netsh advfirewall firewall add rule ...
3. 创建计划任务：schtasks /create ...
4. 注册 Windows Service：sc create ...
5. 启动服务：sc start ...
```

### 3.4 数据库备份实现

**方案：** 在 teacher-server 启动时执行，位于数据库迁移之后

```rust
fn backup_database(db_path: &str) -> Result<()> {
    let backup_dir = format!("data/backup/{}", chrono::Local::now().format("%Y-%m-%d"));
    std::fs::create_dir_all(&backup_dir)?;
    std::fs::copy(db_path, format!("{}/campuslink.db", backup_dir))?;
    // 清理 7 天前备份
    cleanup_old_backups("data/backup", 7)?;
    Ok(())
}
```

### 3.5 日志轮转实现

**方案：** 使用 `tracing-appender` crate 的 `RollingFileAppender`

```rust
use tracing_appender::rolling::{RollingFileAppender, Rotation};

let file_appender = RollingFileAppender::new(Rotation::DAILY, "logs", "teacher-server");
```

学生端同理，但使用 `tracing-subscriber` 的 `fmt` layer 配合 `tracing-appender`。

### 3.6 API Token 认证

**方案：** Axum middleware 层验证 Bearer Token

```rust
async fn auth_middleware(
    State(state): State<AppState>,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Response {
    let auth_header = headers.get("Authorization");
    match auth_header {
        Some(val) if val == format!("Bearer {}", state.admin_token) => next.run(request).await,
        _ => (StatusCode::UNAUTHORIZED, Json(ApiResponse::error(401, "未授权"))).into_response(),
    }
}
```

**Token 生成：**
```rust
fn generate_token() -> String {
    use rand::Rng;
    let token: String = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(32)
        .map(char::from)
        .collect();
    token
}
```

**WebSocket 认证：** 在 query string 中传递 `?token=xxx`，`on_upgrade` 前校验。

### 3.7 前端登录页改造

当前前端 `/login` 为占位页。需改造为：
- Token 输入框（初始 Admin Token 由教师端控制台打印）
- Token 持久化到 localStorage
- Axios 拦截器自动添加 Authorization header
- 未认证时重定向到 /login

## 四、任务拆解

### 任务 1：教师端 Windows Service 注册脚本

- 编写 `teacher-server/install.bat`：sc create + sc config + 防火墙规则
- 编写 `teacher-server/uninstall.bat`：sc delete + 防火墙规则移除
- 修改 `start.bat`：检测是否已安装服务，已安装则启动服务

### 任务 2：学生端开机自启

- 编写 `student-agent/install.bat`：schtasks 创建计划任务
- 编写 `student-agent/uninstall.bat`：schtasks 删除任务
- 更新 `start.bat`：检测自启状态

### 任务 3：Inno Setup 安装器

- 编写 `installer/setup.iss`
- 配置安装界面（欢迎页、许可协议、路径选择、组件选择）
- 安装步骤：释放文件 → 运行 install.bat → 启动服务
- 卸载步骤：停止服务 → 删除任务 → 清理文件

### 任务 4：防火墙放行

- `install.bat` 中添加 netsh 防火墙规则
- 支持 Domain/Private/Public 三网络类型
- 卸载脚本中清理规则

### 任务 5：数据库备份

- `teacher-server/src/infrastructure/backup.rs`：备份逻辑
- 启动流程中插入备份步骤（migration 之后、server start 之前）
- `teacher-server/Cargo.toml` 添加 `chrono` 依赖（已有）

### 任务 6：日志轮转

- `teacher-server/Cargo.toml` 添加 `tracing-appender` 依赖
- `teacher-server/src/main.rs`：替换 `tracing_subscriber::fmt()` 为 `RollingFileAppender`
- `agent-core/src/main.rs`：同样添加日志轮转
- 启动时清理过期日志

### 任务 7：API Token 认证

- `teacher-server/src/api/auth.rs`：auth middleware + token 生成
- `teacher-server/src/main.rs`：首次启动生成 Token，控制台打印
- 注册路由组：management routes 套用 auth layer
- WebSocket 连接增加 token 校验
- 前端 axios 拦截器添加 Authorization header
- 前端 Login.vue 页面改造为 Token 登录

### 任务 8：HTTPS 支持

- `teacher-server/Cargo.toml` 添加 `axum-server`（支持 TLS）
- 配置项增加 `tls_cert_path` / `tls_key_path`
- `generate-cert.bat`：openssl 或 PowerShell 生成自签名证书
- 读取配置决定是否启用 HTTPS

### 任务 9：部署文档

- `docs/deployment-guide.md`：生产部署完整指南
- 拓扑图、安装步骤、运维手册、故障排查
- 随发布包分发的简化版 `README.md`

### 任务 10：编译验证与发布包更新

- 教师端 + 学生端全量编译
- Windows 交叉编译
- 生成安装器 exe
- 发布包整体验证

## 五、验收标准

### 功能验收

- [ ] 教师端作为 Windows Service 运行，开机自启
- [ ] 学生端登录后自动启动 agent-core
- [ ] 安装器一键安装教师端与学生端
- [ ] 防火墙自动放行 8080 端口
- [ ] 每日自动备份 SQLite 数据库
- [ ] 日志按天轮转，过期自动清理
- [ ] 管理 API 需要 Token 认证
- [ ] 前端页面需要登录后使用
- [ ] 部署文档覆盖完整安装与运维流程

### 代码质量验收

- [ ] 所有新增代码通过 `cargo build`
- [ ] 前端通过 `vite build`
- [ ] 错误处理完整
- [ ] 日志记录清晰
- [ ] 不引入安全漏洞（Token 不硬编码、备份文件权限正确）

## 六、风险与应对

### 风险 1：Windows Service 权限不足

**应对：** 使用 LocalSystem 账户运行服务，确保数据库读写和网络监听权限。如学生端需与桌面交互（锁屏），计划任务使用 `/rl highest` 并以当前用户运行。

### 风险 2：MSVC 运行时缺失

**应对：** 安装器中打包 VC++ Redistributable 2015-2022，安装前检测注册表，缺失则静默安装。

### 风险 3：Token 遗忘

**应对：** 首次生成 Token 时打印到控制台并写入 `config/config.toml`。提供 `reset-token.bat` 脚本，停止服务后删除 token 配置并重启以重新生成。

### 风险 4：防火墙规则残留

**应对：** 安装器退出时清理规则。卸载前先执行清理脚本。规则名称带版本前缀便于识别。

### 风险 5：数据库备份占用磁盘

**应对：** 自动清理 7 天前备份。备份前检查磁盘剩余空间（至少保留 100MB）。

---

**文档版本：** v1.0
**生成时间：** 2026-06-19
**状态：** 待用户确认
