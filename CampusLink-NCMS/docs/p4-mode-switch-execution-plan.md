# P4 模式切换基础链路执行基线文档

## 一、目标

本执行基线用于指导教师端和学生端实现模式切换功能，完成 P0 最小业务闭环的最后一环。

## 二、本次产出范围

1. 教师端模式切换 API
2. 教师端 Web 模式控制 UI
3. 学生端模式切换消息处理
4. 模式切换确认与日志
5. WebSocket 消息收发增强

## 三、技术栈

- 教师端：Axum + WebSocket
- 教师端 Web：Vue 3 + Element Plus
- 学生端：Rust + WebSocket
- 协议：Protobuf

## 四、功能清单

### 1. 教师端模式切换 API

**接口**: `POST /api/devices/:id/mode`

请求体：
- targetMode: 目标模式
- effectiveAt: 生效时间
- operatorName: 操作人

响应体：
- commandId: 指令 ID
- deviceId: 设备 ID
- targetMode: 目标模式
- deliveryStatus: 投递状态

### 2. 教师端 WebSocket 广播

- 维护已连接的 WebSocket 客户端
- 支持向指定设备发送模式切换指令
- 支持广播模式切换

### 3. 教师端 Web 模式控制 UI

**设备列表页增强**：
- 每行设备增加"切换模式"按钮
- 模式选择弹窗（开放/授课/考试/锁定）
- 显示当前模式状态

**监控总览页增强**：
- 批量模式切换功能

### 4. 学生端模式处理

- 监听 WebSocket 消息
- 解析模式切换指令
- 应用模式（当前占位）
- 回传确认消息

### 5. 日志记录

- 模式切换操作写入 operation_logs
- 学生端确认写入 operation_logs

## 五、协议消息

### ModeSwitchCommand（教师端 -> 学生端）

```proto
message ModeSwitchCommand {
    MessageMeta meta = 1;
    string command_id = 2;
    string target_device_id = 3;
    ModeType target_mode = 4;
    int64 effective_at = 5;
    string operator_name = 6;
}
```

### ModeSwitchAck（学生端 -> 教师端）

```proto
message ModeSwitchAck {
    MessageMeta meta = 1;
    string command_id = 2;
    string device_id = 3;
    AckCode code = 4;
    string message = 5;
    ModeType applied_mode = 6;
}
```

## 六、目录结构

**教师端新增**：
```
teacher-server/src/
├─ api/
│  ├─ handlers.rs (增强)
│  └─ mod.rs
├─ realtime/
│  ├─ websocket.rs
│  └─ mod.rs
└─ domain/
   ├─ device.rs (增强)
   └─ mod.rs
```

**学生端新增**：
```
student-agent/agent-core/src/
├─ mode.rs
└─ lib.rs (增强)
```

**前端新增**：
```
teacher-web/src/
├─ views/
│  ├─ Devices.vue (增强)
│  └─ Monitor.vue (增强)
└─ api/
   └─ devices.ts (增强)
```

## 七、验收标准

- 教师端可下发模式切换指令
- 学生端可接收并确认模式切换
- 前端可显示当前模式
- 模式切换操作有日志记录
- 端到端联调成功

## 八、后续依赖

- P5: 锁屏功能深化
- P6: 签到与检查
- P7: 硬件监控与告警
