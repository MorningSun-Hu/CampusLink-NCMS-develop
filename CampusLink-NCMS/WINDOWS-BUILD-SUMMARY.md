# CampusLink-NCMS Windows 发布总结

## 一、构建状态

日期：2026-06-15  
平台：Windows x86_64  
工具链：Rust + mingw-w64  
当前版本：v0.7.1 联调修复版

## 二、编译产物

### 教师端

| 文件 | 大小 | 说明 |
|------|------|------|
| `teacher-server.exe` | 15MB | 教师端后端服务，提供 REST API 和 WebSocket |

### 学生端

| 文件 | 大小 | 说明 |
|------|------|------|
| `agent-core.exe` | 11MB | 学生端主进程，负责设备注册、心跳、模式接收 |
| `campus-guard.exe` | 5.5MB | 守护进程框架 |
| `campus-lock.exe` | 5.5MB | 锁屏进程框架 |

发布包位置：

```text
/workspace/CampusLink-NCMS/dist/windows-release/
```

## 三、本轮 Windows 联调修复

本轮解决的问题：

- 教师端配置字段缺失：补充环境变量优先加载路径，减少配置文件依赖。
- SQLite 数据库路径错误：`start.bat` 改为按当前目录动态生成数据库路径。
- SQLite 数据库文件无法打开：启动时自动创建数据库父目录并启用 `create_if_missing(true)`。
- 首次启动缺表：教师端启动时自动执行嵌入式 SQLx 迁移。
- Windows 批处理乱码：`install.bat`、`teacher-server/start.bat`、`student-agent/start.bat` 改为 ASCII 内容。
- WebSocket 心跳响应协议不一致：教师端返回 `heartbeat_ack`，匹配学生端命令枚举。

## 四、实机验证结果

Windows 路径：

```text
C:\Users\Hcy\Desktop\windows-release\windows-release
```

教师端启动验证：

```text
Configuration loaded successfully
Database connection pool created
Database migrations applied
Starting server on 0.0.0.0:8080
```

学生端启动验证：

```text
Device registered successfully
Registration completed
Agent Core ready, current mode: open
Starting WebSocket heartbeat loop
WebSocket connected
Heartbeat sent
Received message: {"type":"heartbeat_ack","ack_code":0,"message":"ok"}
```

当前结论：Windows 教师端服务、SQLite 初始化、学生端注册、WebSocket 连接、30 秒心跳闭环均已验证通过。

## 五、部署方式

### 教师端

```powershell
cd C:\Users\Hcy\Desktop\windows-release\windows-release\teacher-server
.\start.bat
```

### 学生端

```powershell
cd C:\Users\Hcy\Desktop\windows-release\windows-release\student-agent
.\start.bat
```

### 健康检查

```powershell
Invoke-RestMethod http://localhost:8080/api/health
```

### 设备列表验证

```powershell
Invoke-RestMethod http://localhost:8080/api/devices
```

## 六、编译命令

教师端：

```bash
cd teacher-server
cargo build --release --target x86_64-pc-windows-gnu
```

学生端：

```bash
cd student-agent
cargo build --release --target x86_64-pc-windows-gnu
```

发布目录覆盖教师端：

```bash
cp teacher-server/target/x86_64-pc-windows-gnu/release/teacher-server.exe dist/windows-release/teacher-server.exe
cp teacher-server/target/x86_64-pc-windows-gnu/release/teacher-server.exe dist/windows-release/teacher-server/teacher-server.exe
```

## 七、当前功能状态

P0-P5 已完成：

- 设备注册与心跳
- WebSocket 实时通信
- 模式切换基础链路
- 在线状态监控
- 30 秒心跳循环
- 断线重连
- 锁屏框架
- Windows 发布包基础部署

P6 进行中：

- 签到功能
- 检查告警
- 图片上传
- 教师端 Web 管理页

## 八、下一步计划

1. 集成 P6 签到与检查 API。
2. 开发教师端签到管理页与告警处理页。
3. 完善学生端异常上报能力。
4. 增加 Windows Service 或安装器方案。
