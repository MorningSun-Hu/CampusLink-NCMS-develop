# P0 后端接口契约

## 一、目标

为 `teacher-server` 提供 P0 阶段可直接实现的 REST API 契约，覆盖健康检查、系统配置、学生管理、设备管理、监控总览。

## 二、统一约定

### 基础前缀

- `/api`

### 返回格式

成功：

```json
{
  "code": 0,
  "message": "ok",
  "data": {}
}
```

失败：

```json
{
  "code": 4001,
  "message": "error message"
}
```

## 三、接口清单

### 1. `GET /api/health`

用途：服务健康检查。

响应字段：

- `status`
- `service`
- `timestamp`

### 2. `GET /api/system/configs`

用途：读取教师端基础配置。

响应字段：

- `heartbeatIntervalSeconds`
- `defaultMode`
- `teacherFingerprint`
- `deviceRegisterPolicy`

### 3. `GET /api/dashboard/overview`

用途：仪表盘总览。

响应字段：

- `studentCount`
- `registeredDeviceCount`
- `onlineDeviceCount`
- `offlineDeviceCount`
- `currentModeStats`

### 4. `GET /api/monitor/online-devices`

用途：查询在线设备列表。

响应字段：

- `items[].deviceId`
- `items[].deviceCode`
- `items[].deviceName`
- `items[].studentName`
- `items[].seatNo`
- `items[].onlineStatus`
- `items[].currentMode`
- `items[].lastSeenAt`

### 5. `GET /api/students`

用途：学生列表查询。

查询参数：

- `keyword`
- `status`
- `page`
- `pageSize`

响应字段：

- `items[].id`
- `items[].studentNo`
- `items[].name`
- `items[].seatNo`
- `items[].status`
- `pagination`

### 6. `POST /api/students`

用途：创建学生。

请求体：

- `studentNo`
- `name`
- `password`
- `seatNo`

响应字段：

- `id`
- `studentNo`
- `name`
- `seatNo`
- `status`

### 7. `PUT /api/students/:id`

用途：更新学生。

请求体：

- `name`
- `password`
- `seatNo`
- `status`

响应字段：

- `id`
- `studentNo`
- `name`
- `seatNo`
- `status`

### 8. `GET /api/devices`

用途：设备列表查询。

查询参数：

- `onlineStatus`
- `currentMode`
- `keyword`
- `page`
- `pageSize`

响应字段：

- `items[].id`
- `items[].deviceCode`
- `items[].deviceName`
- `items[].hostname`
- `items[].ipAddress`
- `items[].registerStatus`
- `items[].onlineStatus`
- `items[].currentMode`
- `items[].lastSeenAt`

### 9. `GET /api/devices/:id`

用途：设备详情查询。

响应字段：

- `id`
- `deviceCode`
- `deviceName`
- `hostname`
- `machineFingerprint`
- `ipAddress`
- `macAddress`
- `registerStatus`
- `onlineStatus`
- `currentMode`
- `student`

### 10. `POST /api/devices/register`

用途：设备通过 HTTP 辅助完成首次注册。

请求体：

- `deviceCode`
- `machineFingerprint`
- `hostname`
- `ipAddress`
- `macAddress`
- `agentVersion`

响应字段：

- `deviceId`
- `teacherFingerprint`
- `initialMode`
- `heartbeatIntervalSeconds`

### 11. `POST /api/devices/:id/mode`

用途：下发单设备模式切换。

请求体：

- `targetMode`
- `effectiveAt`
- `operatorName`

响应字段：

- `commandId`
- `deviceId`
- `targetMode`
- `deliveryStatus`

## 四、编码要求

- 接口名称与字段名在前后端保持一致
- 所有列表接口支持分页参数
- 学生密码只在写入时出现，读取接口不返回明文或哈希
- 模式切换成功后写入 `operation_logs`
