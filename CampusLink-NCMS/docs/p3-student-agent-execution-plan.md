# P3 学生端 Agent 执行基线文档

## 一、目标

本执行基线用于指导学生端 Agent 主进程实现，确保学生端可以完成设备注册、建立心跳、接收模式切换指令。

## 二、本次产出范围

1. `agent-core` 主进程框架
2. 设备注册逻辑
3. 心跳发送逻辑
4. 模式切换消息处理
5. 本地配置与持久化
6. `campus-guard` 守护进程（基础版）

## 三、技术栈

- Rust + Tokio
- windows-rs (Windows API)
- WebSocket 客户端
- Protobuf 协议解析
- 本地配置存储

## 四、功能清单

### 1. 设备注册

- 采集本机信息（主机名、IP、MAC、机器指纹）
- 调用教师端 `POST /api/devices/register`
- 保存返回的设备 ID 和教师指纹
- 注册失败时重试

### 2. 心跳逻辑

- 连接 WebSocket `/ws`
- 定时发送心跳消息（15 秒间隔）
- 接收心跳响应
- 断线重连

### 3. 模式切换

- 接收模式切换指令
- 解析并应用模式
- 回传确认消息

### 4. 本地配置

- 存储教师端地址
- 存储设备 ID
- 存储当前模式
- 配置文件加密（预留）

## 五、目录结构

```
student-agent/
├─ Cargo.toml
├─ agent-core/
│  ├─ src/
│  │  ├─ main.rs
│  │  ├─ config.rs
│  │  ├─ register.rs
│  │  ├─ heartbeat.rs
│  │  ├─ mode.rs
│  │  └─ proto/
│  ├─ Cargo.toml
├─ campus-guard/
│  ├─ src/
│  │  └─ main.rs
│  ├─ Cargo.toml
└─ campus-lock/
   └─ src/
      └─ main.rs (占位)
```

## 六、协议消息

使用 P0 定义的 Protobuf：
- `RegisterRequest` / `RegisterResponse`
- `HeartbeatRequest` / `HeartbeatResponse`
- `ModeSwitchCommand` / `ModeSwitchAck`

## 七、验证标准

- `agent-core` 可编译运行
- 首次运行自动注册
- 心跳定时发送
- 模式切换可接收
- 守护进程可拉起主进程

## 八、后续依赖

- P4: 锁屏功能
- P5: 双进程互保
- P6: 自定义进程守护
