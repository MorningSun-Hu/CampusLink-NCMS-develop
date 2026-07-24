# CampusLink-NCMS 生产部署指南

## 一、部署拓扑

```
                    +--------------------------+
                    |    教师机 (Teacher PC)     |
                    |    Windows 10/11 x64      |
                    |    固定 IP: 192.168.x.x   |
                    |                          |
                    |  teacher-server.exe      |
                    |  (Windows Service)        |
                    |  端口: TCP 8080           |
                    |  Web UI + API + WS        |
                    +------------+-------------+
                                 |
               LAN (同一交换机/路由器)
                                 |
          +----------------------+----------------------+
          |                      |                      |
+---------+--------+  +----------+-------+  +----------+-------+
|  学生机 1 (PC-01) |  | 学生机 2 (PC-02) |  | 学生机 N (PC-N) |
|  Windows 10/11    |  |  Windows 10/11  |  |  Windows 10/11  |
|                   |  |                 |  |                 |
|  agent-core.exe   |  | agent-core.exe  |  | agent-core.exe  |
|  campus-guard.exe |  | campus-guard    |  | campus-guard    |
|  campus-lock.exe  |  | campus-lock     |  | campus-lock     |
+-------------------+  +-----------------+  +-----------------+
```

## 二、环境要求

| 项目 | 最低要求 |
|------|----------|
| 操作系统 | Windows 10/11 x64, Windows Server 2016+ |
| 内存 | 教师机 4GB, 学生机 2GB |
| 磁盘 | 教师机 1GB (含数据库+日志), 学生机 100MB |
| 网络 | 同一局域网, 教师机固定 IP, 互 Ping 可达 |
| 运行时 | MSVC++ 2015-2022 Redistributable (VCRUNTIME140.dll) |

## 三、安装步骤

### 3.1 前置检查

```powershell
# 检查 MSVC 运行时
where VCRUNTIME140.dll

# 如未安装, 下载运行:
# https://aka.ms/vs/17/release/vc_redist.x64.exe
```

### 3.2 解压发布包

将 `CampusLink-NCMS-windows-x64.zip` 解压到目标目录, 例如:

```
C:\CampusLink-NCMS\
```

### 3.3 教师端安装

**方式一: 统一安装器 (推荐)**

以管理员身份运行:

```bat
# 右键 install.bat → "以管理员身份运行"
```

安装器将自动完成:
1. 创建 `data/` 和 `logs/` 目录
2. 注册 `CampusLinkTeacher` Windows Service (开机自启)
3. 添加防火墙入站规则 (TCP 8080)
4. 创建默认学生端配置

**方式二: 手动安装**

```bat
# 1. 防火墙放行
cd teacher-server
.\install-firewall.bat

# 2. 注册服务
.\install-service.bat

# 3. 启动服务
sc start CampusLinkTeacher
```

**方式三: 手动启动 (开发调试)**

```bat
# 不以服务运行, 直接前台启动
cd teacher-server
.\start.bat
```

### 3.4 学生端安装

**方式一: 统一安装器**

统一安装器自动配置 `CampusLink Student Agent` 计划任务 (用户登录时启动)。

**方式二: 手动配置**

```bat
# 1. 编辑配置文件
notepad student-agent\config.json
```

```json
{
  "teacher_server_url": "http://192.168.1.100:8080",
  "device_name": "PC-01",
  "heartbeat_interval_secs": 30
}
```

```bat
# 2. 配置开机自启
cd student-agent
.\install-autostart.bat

# 3. 手动启动
.\start.bat
```

> 将 `192.168.1.100` 替换为教师机实际 IP 地址。

### 3.5 多台学生机批量部署

```powershell
# 复制整个 student-agent 目录到每台学生机
# 逐台修改 config.json 中的 device_name

# 示例: 修改 30 台机器的 device_name
$ip = "192.168.1.100"
1..30 | ForEach-Object {
    $name = "PC-{0:D2}" -f $_
    $dir = "\\PC-$_\C$\CampusLink-NCMS\student-agent"
    $json = @{
        teacher_server_url = "http://${ip}:8080"
        device_name = $name
        heartbeat_interval_secs = 30
    } | ConvertTo-Json
    New-Item -Path $dir -ItemType Directory -Force
    Set-Content -Path "$dir\config.json" -Value $json
}
```

## 四、验证安装

### 4.1 检查教师端服务

```powershell
sc query CampusLinkTeacher
# STATE: RUNNING

Invoke-RestMethod http://localhost:8080/api/health
# {"code":0,"data":{"status":"ok"}}
```

### 4.2 检查防火墙规则

```powershell
netsh advfirewall firewall show rule name="CampusLink Teacher Server (8080)"
# Enabled: Yes
```

### 4.3 检查学生端

学生机启动后, 在教师端浏览器访问 `http://localhost:8080`, 使用默认账号登录:

- 用户名: `admin`
- 密码: `admin123`

在设备管理页面 (`/devices`) 应能看到学生机上线。

### 4.4 端到端验证

| 验证项 | 操作 | 期望结果 |
|--------|------|----------|
| 教师端 Web 访问 | 浏览器打开 http://localhost:8080 | 显示登录页 |
| 管理员登录 | admin / admin123 | 跳转仪表盘 |
| 学生机注册 | 学生端启动 agent-core | 设备列表出现新设备 |
| 实时监控 | 查看仪表盘在线数 | 在线设备数 > 0 |
| 模式切换 | 在设备页面切换模式 | 学生端进入对应模式 |
| 锁屏 | 点击锁屏按钮 | 学生端全屏锁屏 |
| 签到 | 学生端自动签到 | 签到记录页有记录 |
| 硬件快照 | 学生端上报 | 硬件页有快照详情 |
| 日志查询 | 查看日志中心 | 有操作日志 |

## 五、日常运维

### 5.1 查看服务状态

```powershell
# 教师端
sc query CampusLinkTeacher
sc stop CampusLinkTeacher
sc start CampusLinkTeacher

# 查看日志
type teacher-server\logs\teacher-server-YYYY-MM-DD.log
```

### 5.2 数据库备份

```powershell
# 手动备份
copy teacher-server\data\campuslink.db "D:\backups\campuslink-%DATE:/=-%.db"

# 系统已配置自动备份: 每日备份到 data/backup/YYYY-MM-DD/
# 保留最近 7 天, 过期自动清理
```

### 5.3 恢复数据库

```powershell
# 停止服务
sc stop CampusLinkTeacher

# 替换数据库文件
copy teacher-server\data\backup\2026-07-03\campuslink.db teacher-server\data\campuslink.db

# 启动服务
sc start CampusLinkTeacher
```

### 5.4 修改超级密码

登录 Web 界面 → 系统设置 → 修改锁屏密码。修改后系统自动通过 WebSocket 广播到所有在线学生端, 实时生效。

### 5.5 管理管理员账号

```powershell
# 当前仅支持通过 API 操作
# 查看管理员列表 (需 bearer token)
Invoke-RestMethod -Uri http://localhost:8080/api/auth/users -Headers @{Authorization="Bearer $token"}
```

### 5.6 更新教师端服务

```powershell
# 1. 停止服务
sc stop CampusLinkTeacher

# 2. 替换可执行文件
copy /Y new\teacher-server.exe teacher-server\

# 3. 替换前端静态文件
copy /Y new\static\* teacher-server\static\

# 4. 启动服务
sc start CampusLinkTeacher
```

### 5.7 更新学生端

```powershell
# 1. 结束计划任务
schtasks /end /tn "CampusLink Student Agent"

# 2. 替换可执行文件
copy /Y new\agent-core.exe student-agent\
copy /Y new\campus-guard.exe student-agent\
copy /Y new\campus-lock.exe student-agent\

# 3. 重新启动
schtasks /run /tn "CampusLink Student Agent"
```

## 六、故障排查

### 教师端服务无法启动

```powershell
# 检查端口占用
netstat -ano | findstr :8080

# 检查配置文件
type teacher-server\config\config.toml

# 检查数据库权限
icacls teacher-server\data

# 查看 Windows 事件日志
Get-EventLog -LogName Application -Source "CampusLinkTeacher" -Newest 10
```

### 学生端无法连接

```powershell
# 检查网络连通性
ping 192.168.1.100

# 检查端口可达
Test-NetConnection 192.168.1.100 -Port 8080

# 检查防火墙
netsh advfirewall firewall show rule name="CampusLink Teacher Server (8080)"

# 查看学生端日志
type student-agent\logs\agent-core-YYYY-MM-DD.log
```

### 常见问题速查

| 现象 | 原因 | 解决 |
|------|------|------|
| `unable to open database file` | data 目录不存在或权限不足 | 手动创建 `data/` 目录 |
| `no such table` | 数据库迁移未执行 | 重启服务, 迁移会在启动时自动执行 |
| `connection refused` | 教师端未启动或防火墙拦截 | 检查服务状态和防火墙规则 |
| `401 Unauthorized` | Token 过期 | 重新登录 Web 界面 |
| 锁屏后无法解锁 | 超级密码错误或进程未启动 | 确认密码正确; 检查 campus-lock.exe 存在 |
| 设备列表无学生机 | 学生端未注册或网络不通 | 检查学生端日志和网络连通性 |
| 仪表盘数据为 0 | 数据库未初始化 | 等待学生端注册, 或检查迁移日志 |
| 日志文件过大 | 日志轮转未启用 | 检查 tracing-appender 配置 |

## 七、安全建议

### 7.1 网络隔离

- 教师机和学生机使用独立 VLAN 或交换机
- 禁止学生机访问互联网 (除非教学需要)
- 教师机 8080 端口仅对局域网开放

### 7.2 密码管理

- 首次登录后立即修改默认管理员密码 (admin/admin123)
- 定期轮换锁屏超级密码
- 不要将密码存储在共享目录或明文文件中

### 7.3 数据库安全

- 定期备份 `campuslink.db` 到安全位置
- 考虑启用 SQLCipher 加密 (需重新编译)
- 限制 `data/` 目录的文件系统访问权限

### 7.4 日志审计

- 定期查看操作日志 (`/logs`) 识别异常行为
- 保留足够长的日志周期 (当前 14 天)
- 对关键操作 (删除设备、修改密码) 设置告警

## 八、可选: 启用 HTTPS

### 8.1 生成自签名证书

```bat
cd teacher-server
.\generate-cert.bat
```

### 8.2 配置 TLS

编辑 `teacher-server/config/config.toml`:

```toml
[tls]
enabled = true
cert_path = "config/certs/cert.pem"
key_path = "config/certs/key.pem"
```

### 8.3 注意事项

- 自签名证书在浏览器中会显示不安全警告, 生产环境建议使用 CA 签发证书
- 启用 HTTPS 后学生端 `teacher_server_url` 需改为 `https://...`

---

文档版本: v1.0
适用版本: CampusLink-NCMS v0.9.0+
最后更新: 2026-07-04
