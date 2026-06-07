# P1 设备注册与心跳链路执行基线文档

## 一、目标

本执行基线用于指导教师端服务实现设备注册与心跳链路，确保学生端 Agent 可以完成注册、建立心跳、维持在线状态。

## 二、本次产出范围

1. 设备注册 REST 接口
2. WebSocket 实时通信入口
3. 心跳消息处理
4. 在线状态管理
5. 日志记录

## 三、接口清单

### 1. POST /api/devices/register

请求体：
- deviceCode
- machineFingerprint
- hostname
- ipAddress
- macAddress
- agentVersion

响应体：
- deviceId
- teacherFingerprint
- initialMode
- heartbeatIntervalSeconds

### 2. GET /api/devices

查询参数：
- onlineStatus
- currentMode
- keyword
- page
- pageSize

响应体：
- items[].id/.deviceCode/.deviceName/.registerStatus/.onlineStatus/.currentMode/.lastSeenAt
- pagination

### 3. GET /api/monitor/online-devices

响应体：
- items[].deviceId/.deviceCode/.deviceName/.studentName/.onlineStatus/.currentMode/.lastSeenAt

### 4. WebSocket /ws

连接建立后：
- 客户端发送心跳消息
- 服务端响应心跳确认
- 服务端定期广播模式切换指令

## 四、数据库操作

### 设备注册

- 向 `student_devices` 插入记录
- 生成 UUID 作为 deviceId
- 写入注册成功日志到 `operation_logs`

### 心跳处理

- 更新 `student_devices.last_seen_at`
- 更新 `student_devices.online_status = 'online'`
- 记录心跳日志

### 离线判定

- 定期扫描超过心跳间隔的设备
- 更新 `online_status = 'offline'`

## 五、错误码定义

- 200: 成功
- 400: 参数错误
- 409: 设备已注册
- 500: 服务器错误

## 六、验收标准

- 设备注册接口正常工作
- WebSocket 建立连接
- 心跳定时更新
- 在线状态监控准确
- 日志记录完整
