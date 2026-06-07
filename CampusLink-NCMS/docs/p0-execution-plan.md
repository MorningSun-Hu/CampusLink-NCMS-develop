# CampusLink-NCMS P0 执行清单

## 一、P0 目标

P0 需要打通最小业务闭环，确保教师端服务、教师端 Web、学生端 Agent 可以围绕注册、心跳、在线状态、模式控制完成首次联动。

P0 完成后应达到以下结果：

- 教师端服务可启动并连接数据库
- 教师端 Web 可访问并调用教师端服务 API
- 学生端 Agent 可注册到教师端并建立心跳
- 教师端可查看在线状态并下发基础模式切换

## 二、P0 交付物

### 1. 工程交付物

- `teacher-server` 可编译运行
- `teacher-web` 可本地开发预览
- `student-agent` 三个子进程可独立编译
- `proto` 协议文件与生成脚本可执行

### 2. 业务交付物

- 设备注册链路
- 心跳保活链路
- 在线状态展示链路
- 模式切换基础链路

## 三、P0 数据表清单

### 必需表

- `students`
- `student_devices`
- `system_configs`
- `operation_logs`

### 建议字段方向

#### `students`

- `id`
- `student_no`
- `name`
- `password_hash`
- `seat_no`
- `status`
- `created_at`
- `updated_at`

#### `student_devices`

- `id`
- `student_id`
- `device_code`
- `device_name`
- `machine_fingerprint`
- `ip_address`
- `mac_address`
- `register_status`
- `last_seen_at`
- `current_mode`
- `created_at`
- `updated_at`

#### `system_configs`

- `id`
- `config_key`
- `config_value`
- `scope`
- `updated_at`

#### `operation_logs`

- `id`
- `log_type`
- `operator`
- `target_id`
- `content`
- `created_at`

## 四、P0 协议清单

### 必需消息类型

- `REG_REQ`
- `REG_RESP`
- `HEARTBEAT`
- `MODE_SWITCH`
- `MODE_SWITCH_ACK`

### 协议文件建议

- `proto/registration.proto`
- `proto/heartbeat.proto`
- `proto/mode.proto`
- `proto/common.proto`

### 协议字段方向

#### `REG_REQ`

- 设备编号
- 机器指纹
- 主机名
- IP
- MAC
- Agent 版本
- 时间戳

#### `REG_RESP`

- 注册结果
- 分配设备 ID
- 教师端指纹
- 初始模式
- 心跳间隔

#### `HEARTBEAT`

- 设备 ID
- 当前模式
- 在线状态
- Agent 版本
- 时间戳

#### `MODE_SWITCH`

- 设备 ID 或广播范围
- 目标模式
- 生效时间
- 操作人

## 五、P0 后端接口清单

### 系统接口

- `GET /api/health`
- `GET /api/system/configs`

### 设备接口

- `GET /api/devices`
- `GET /api/devices/:id`
- `POST /api/devices/register`
- `POST /api/devices/:id/mode`

### 学生接口

- `GET /api/students`
- `POST /api/students`
- `PUT /api/students/:id`

### 监控接口

- `GET /api/dashboard/overview`
- `GET /api/monitor/online-devices`

## 六、P0 前端页面清单

### 必需页面

- 登录页
- 仪表盘
- 学生列表页
- 设备列表页
- 监控总览页

### 必需组件

- 顶部导航
- 侧边菜单
- 在线状态卡片
- 设备表格
- 模式切换弹窗

## 七、P0 学生端 Agent 任务

### `agent-core`

- 本地配置加载
- 注册请求发送
- WebSocket 建链
- 定时心跳发送
- 模式切换消息处理

### `campus-guard`

- 监控 `agent-core` 存活
- 异常退出后拉起

### `campus-lock`

- 预留模式锁定入口
- 当前阶段提供最小占位实现

## 八、P0 开发顺序

1. 定义数据库表与迁移
2. 定义 Protobuf 协议与生成脚本
3. 完成 `teacher-server` 配置加载、数据库接入、基础路由
4. 完成设备注册与心跳接收
5. 完成 `teacher-web` 登录页、仪表盘、设备列表、监控页
6. 完成 `agent-core` 注册、心跳、模式接收
7. 完成最小联调与日志记录

## 九、P0 验收标准

- 教师端服务可成功启动
- 教师端 Web 可展示在线设备数量
- 学生端 Agent 首次运行后可完成注册
- 教师端在 30 秒内可看到心跳更新时间变化
- 教师端下发模式切换后，学生端可收到并回传确认
