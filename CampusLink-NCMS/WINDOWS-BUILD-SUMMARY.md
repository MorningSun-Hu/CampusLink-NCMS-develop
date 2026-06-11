# CampusLink-NCMS Windows 发布总结

## 🎉 编译成功

**日期**: 2026-06-11  
**平台**: Windows x86_64 (64 位)  
**工具链**: Rust + mingw-w64

---

## ✅ 编译产物

### 学生端 (Student Agent)

| 文件 | 大小 | 说明 |
|------|------|------|
| `agent-core.exe` | 11MB | 主进程 - 设备注册、心跳、模式接收 |
| `campus-guard.exe` | 5.5MB | 守护进程 - 监控主进程 |
| `campus-lock.exe` | 5.5MB | 锁屏进程 - 框架版本 |

### 教师端 (Teacher Server)

| 文件 | 大小 | 说明 |
|------|------|------|
| `teacher-server.exe` | 15MB | 后端服务 - REST API + WebSocket |

**总计**: 4 个可执行文件，共 37MB

---

## 📦 发布包位置

```
/workspace/CampusLink-NCMS/dist/windows-release/
├─ teacher-server/
│  └─ teacher-server.exe (15MB)
├─ student-agent/
│  ├─ agent-core.exe (11MB)
│  ├─ campus-guard.exe (5.5MB)
│  └─ campus-lock.exe (5.5MB)
└─ README_windows.md
```

---

## 🚀 部署到 Windows

### 方式 1: 直接复制

将 `dist/windows-release/` 目录复制到 Windows 机器：

```powershell
# Windows 上执行
xcopy \\linux-server\workspace\CampusLink-NCMS\dist\windows-release C:\CampusLink\ /E /Y
```

### 方式 2: 打包下载

```bash
cd /workspace/CampusLink-NCMS/dist
tar -czf windows-release-v0.6.0.tar.gz windows-release/
```

然后下载 `windows-release-v0.6.0.tar.gz`。

---

## 💻 功能状态

### P0-P5 (已实现)

✅ **核心功能**
- 设备注册与心跳
- WebSocket 实时通信
- 模式切换（5 种模式）
- 在线状态监控
- 断线重连

✅ **技术栈**
- 教师端：Rust + Axum + SQLx + SQLite
- 学生端：Rust + Tokio + WebSocket
- 通信协议：JSON（可升级到 Protobuf）

### P6 (待集成)

⬜ **签到功能**
- API 已编写，待编译
- 前端页面待开发

⬜ **检查告警**
- 异常报告 API 已编写
- 图片上传 API 已编写
- 前端页面待开发

---

## 🔍 验证清单

在 Windows 上部署后验证：

### 教师端验证

1. **启动服务**
   ```powershell
   cd C:\CampusLink\teacher-server
   .\teacher-server.exe
   ```

2. **健康检查**
   ```bash
   curl http://localhost:8080/api/health
   # 应返回：ok
   ```

3. **数据库创建**
   ```powershell
   Test-Path data\campuslink.db
   # 应返回：True
   ```

### 学生端验证

1. **配置教师端地址**
   编辑 `config/config.json`

2. **启动主进程**
   ```powershell
   cd C:\CampusLink\student-agent
   .\agent-core.exe
   ```

3. **查看设备注册**
   - 检查日志：`type logs\agent-core.log`
   - 应看到注册成功消息

4. **验证 WebSocket**
   - 日志中应看到心跳发送和接收

---

## 🔧 故障排查

### 问题 1: "dll not found" 错误

**解决方案**: 安装 Visual C++ 可再发行组件

下载地址：https://aka.ms/vs/17/release/vc_redist.x64.exe

### 问题 2: 无法连接教师端

**检查步骤**:
1. 确认教师端正在运行
2. 检查防火墙设置
3. 验证 IP 地址配置
4. 测试网络连通性

### 问题 3: 学生端注册失败

**解决方案**:
```powershell
# 删除配置重新注册
Remove-Item config\config.json
.\agent-core.exe
```

---

## 📊 跨平台编译说明

### 使用工具

- **Rust**: 编译器
- **mingw-w64**: GNU 工具链
- **target**: `x86_64-pc-windows-gnu`

### 编译命令

```bash
# 学生端
cd student-agent
cargo build --release --target x86_64-pc-windows-gnu

# 教师端
cd teacher-server
cargo build --release --target x86_64-pc-windows-gnu
```

### 编译产物

```
target/x86_64-pc-windows-gnu/release/*.exe
```

---

## 📈 下一步计划

1. **集成 P6 功能**
   - 编译签到模块
   - 编译告警模块

2. **前端开发**
   - 签到管理页面
   - 告警处理页面

3. **端到端联调**
   - 验证签到流程
   - 验证告警流程

4. **性能优化**
   - 内存使用优化
   - 启动速度优化

---

**发布状态**: ✅ 可用  
**测试状态**: ⚠️ 待 Windows 端验证  
**文档状态**: ✅ 完整

---

构建完成时间：2026-06-11
