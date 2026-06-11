# P6 执行基线文档：签到与检查流程

## 一、阶段目标

在 P0-P5 打通 WebSocket 心跳循环与锁屏框架的基础上，实现完整的课堂签到与检查流程，包括：
1. 学生签到流程（发起、记录、统计）
2. 设备检查流程（卫生检查、设备检查）
3. 拍照上传接口（课堂现场记录）
4. 学生登录功能（学生端身份认证）

## 二、功能范围

### 6.1 学生签到流程

**功能描述：**
- 学生端 Agent 启动后自动发起签到（或手动触发）
- 教师端记录签到状态（时间、设备、学生信息）
- 教师端 Web 显示签到统计（应到/实到/未到名单）
- 支持补签操作

**验收标准：**
- [ ] 学生端可发起签到请求
- [ ] 教师端正确记录签到信息
- [ ] 教师端 Web 显示签到统计
- [ ] 支持查看未到学生名单
- [ ] 教师可手动补签
- [ ] 签到记录可导出（CSV/Excel）

### 6.2 设备检查流程

**功能描述：**
- 卫生检查：键盘、鼠标、显示器、桌面清洁度
- 设备检查：摄像头、麦克风、耳机、投影仪状态
- 检查结果分级：正常/异常/缺失
- 异常情况记录与告警

**验收标准：**
- [ ] 学生端可发起检查请求
- [ ] 支持多项检查（卫生、设备）
- [ ] 检查结果分级记录
- [ ] 异常情况标记与告警
- [ ] 教师端可查看检查统计
- [ ] 检查记录可导出

### 6.3 告警与图片上传

**功能描述：**
- 学生端报告异常（卫生/设备异常）时，触发教师端告警
- 教师在告警处理界面可从本地上传图片
- 图片与告警记录关联存储
- 支持多张图片上传

**调整说明：**
- ❌ 学生端不实现摄像头调用
- ❌ 教师端不实现摄像头调用
- ✅ 教师端保留图片上传功能（从本地文件选择）
- ✅ 告警触发后，教师在教师端处理时上传图片
- ✅ 图片与告警记录关联

**验收标准：**
- [ ] 学生端可报告异常情况
- [ ] 教师端收到告警通知
- [ ] 教师端可查看告警列表
- [ ] 告警处理界面可上传图片
- [ ] 支持多张图片上传
- [ ] 图片与告警记录关联
- [ ] 图片可预览

### 6.4 学生登录功能

**功能描述：**
- 学生端登录界面（命令行或简单 GUI）
- 账号密码验证（调用教师端 API）
- 登录状态持久化（JSON 配置文件）
- 支持自动登录（记住登录状态）

**验收标准：**
- [ ] 学生端可输入账号密码
- [ ] 教师端验证账号密码
- [ ] 登录成功返回会话 token
- [ ] 登录状态持久化存储
- [ ] 支持自动登录（7 天有效期）
- [ ] 支持退出登录

## 三、技术实现方案

### 3.1 数据库设计

**新增表结构：**

```sql
-- 签到记录表
CREATE TABLE attendance_records (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL,
    device_id INTEGER NOT NULL,
    check_in_time DATETIME NOT NULL,
    check_out_time DATETIME,
    status TEXT NOT NULL,  -- present, late, absent, leave
    remarks TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (student_id) REFERENCES students(id),
    FOREIGN KEY (device_id) REFERENCES student_devices(id)
);

-- 检查记录表
CREATE TABLE inspection_records (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    device_id INTEGER NOT NULL,
    student_id INTEGER,
    inspection_type TEXT NOT NULL,  -- hygiene, equipment
    item_name TEXT NOT NULL,        -- keyboard, mouse, camera, etc.
    status TEXT NOT NULL,          -- normal, abnormal, missing
    description TEXT,
    photo_url TEXT,
    inspector TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (device_id) REFERENCES student_devices(id),
    FOREIGN KEY (student_id) REFERENCES students(id)
);

-- 照片存储表
CREATE TABLE photos (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER,
    device_id INTEGER NOT NULL,
    file_path TEXT NOT NULL,
    file_size INTEGER,
    mime_type TEXT,
    upload_time DATETIME DEFAULT CURRENT_TIMESTAMP,
    description TEXT,
    FOREIGN KEY (student_id) REFERENCES students(id),
    FOREIGN KEY (device_id) REFERENCES student_devices(id)
);
```

### 3.2 教师端 API 设计

**签到 API：**

```rust
// POST /api/attendance/check-in
// 学生签到
pub struct CheckInRequest {
    pub device_id: String,
    pub student_id: Option<String>,
    pub timestamp: u64,
}

pub struct CheckInResponse {
    pub record_id: i32,
    pub status: String,
}

// GET /api/attendance/statistics?date=2024-01-01
// 获取签到统计
pub struct AttendanceStatistics {
    pub total: i32,
    pub present: i32,
    pub late: i32,
    pub absent: i32,
    pub leave: i32,
    pub records: Vec<AttendanceRecord>,
}

// POST /api/attendance/retroactive
// 补签
pub struct RetroactiveRequest {
    pub student_id: i32,
    pub device_id: i32,
    pub check_in_time: String,
    pub remarks: Option<String>,
}
```

**告警 API：**

```rust
// POST /api/inspection/submit
// 提交检查/报告异常
pub struct InspectionSubmitRequest {
    pub device_id: String,
    pub inspection_type: String,  // hygiene, equipment
    pub items: Vec<InspectionItem>,
    pub is_abnormal: bool,         // 是否异常报告
}

pub struct InspectionItem {
    pub item_name: String,
    pub status: String,  // normal, abnormal, missing
    pub description: Option<String>,
}

// GET /api/alerts/list
// 获取告警列表
pub struct AlertListResponse {
    pub total: i32,
    pub pending: i32,
    pub processed: i32,
    pub alerts: Vec<AlertRecord>,
}

pub struct AlertRecord {
    pub alert_id: i32,
    pub device_id: i32,
    pub student_id: Option<i32>,
    pub alert_type: String,  // hygiene, equipment
    pub description: String,
    pub status: String,      // pending, processing, resolved
    pub created_at: String,
    pub photo_urls: Vec<String>,
}

// POST /api/alerts/:id/resolve
// 处理告警（上传图片）
pub struct ResolveAlertRequest {
    pub status: String,
    pub remarks: Option<String>,
    pub photo_ids: Vec<i32>,  // 关联的照片 ID
}
```

**图片上传 API：**

```rust
// POST /api/photos/upload - multipart/form-data
pub struct PhotoUploadRequest {
    pub file: TempFile,
    pub alert_id: Option<i32>,   // 关联告警 ID（可选）
    pub description: Option<String>,
}

// GET /api/photos/:id
// 获取照片（二进制）
```

### 3.3 学生端实现

**签到模块：**

```rust
// student-agent/agent-core/src/attendance.rs
pub struct AttendanceClient {
    config: Config,
    client: reqwest::Client,
}

impl AttendanceClient {
    pub async fn check_in(&self) -> Result<CheckInResponse>;
    pub async fn check_out(&self) -> Result<()>;
}
```

**检查与告警模块：**

```rust
// student-agent/agent-core/src/inspection.rs
pub struct InspectionClient {
    config: Config,
}

impl InspectionClient {
    pub async fn submit_hygiene_check(&self, items: Vec<HygieneItem>) -> Result<()>;
    pub async fn submit_equipment_check(&self, items: Vec<EquipmentItem>) -> Result<()>;
    pub async fn report_abnormal(&self, alert_type: &str, description: &str) -> Result<()>;
}
```

**学生登录模块：**

```rust
// student-agent/agent-core/src/student_auth.rs
pub struct StudentAuth {
    config: Config,
}

impl StudentAuth {
    pub async fn login(&mut self, username: &str, password: &str) -> Result<LoginResponse>;
    pub async fn logout(&mut self) -> Result<()>;
    pub fn is_logged_in(&self) -> bool;
    pub fn get_current_student(&self) -> Option<StudentInfo>;
}
```

**说明：**
- 学生端不实现摄像头调用
- 学生端报告异常后，由教师在教师端处理时上传图片

### 3.4 教师端 Web 前端

**新增页面：**

```
/src/views/
├─ Attendance.vue          # 签到管理页
├─ Inspection.vue          # 检查管理页
└─ Photos.vue              # 照片浏览页

/src/components/
├─ attendance/
│  ├─ StatisticsCard.vue   # 签到统计卡片
│  ├─ AttendanceTable.vue  # 签到明细表
│  └─ RetroactiveDialog.vue # 补签弹窗
├─ inspection/
│  ├─ InspectionForm.vue   # 检查表单
│  └─ InspectionTable.vue  # 检查记录表
└─ photos/
   ├─ PhotoUpload.vue      # 照片上传组件
   └─ PhotoGallery.vue     # 照片浏览
```

## 四、任务拆解

### 任务 1：数据库迁移

**文件清单：**
- `teacher-server/migrations/0006_create_attendance_records.sql`
- `teacher-server/migrations/0007_create_inspection_records.sql`
- `teacher-server/migrations/0008_create_photos.sql`

**实现步骤：**
1. 编写迁移 SQL 文件
2. 执行迁移验证
3. 添加种子数据（可选）

### 任务 2：教师端签到 API

**文件清单：**
- `teacher-server/src/api/attendance.rs`（新建）
- `teacher-server/src/api/mod.rs`（更新）
- `teacher-server/src/domain/attendance.rs`（新建）

**实现步骤：**
1. 定义请求/响应结构
2. 实现签到逻辑
3. 实现统计查询
4. 实现补签功能
5. 添加单元测试

### 任务 3：教师端检查 API

**文件清单：**
- `teacher-server/src/api/inspection.rs`（新建）
- `teacher-server/src/domain/inspection.rs`（新建）

**实现步骤：**
1. 定义检查类型枚举
2. 实现检查提交逻辑
3. 实现检查记录查询
4. 实现异常告警

### 任务 4：教师端告警与图片上传 API

**文件清单：**
- `teacher-server/src/api/alerts.rs`（新建）
- `teacher-server/src/api/photos.rs`（新建）
- `teacher-server/src/storage.rs`（新建）

**实现步骤：**
1. 实现告警列表查询 API
2. 实现告警处理 API（更新状态）
3. 实现 multipart 文件上传处理
4. 实现照片存储（本地文件系统）
5. 实现照片与告警关联

### 任务 5：学生端签到模块

**文件清单：**
- `student-agent/agent-core/src/attendance.rs`（新建）
- `student-agent/agent-core/src/main.rs`（更新）

**实现步骤：**
1. 实现签到 API 调用
2. 集成到启动流程
3. 支持命令行触发

### 任务 6：学生端检查与告警模块

**文件清单：**
- `student-agent/agent-core/src/inspection.rs`（新建）

**实现步骤：**
1. 定义检查项结构
2. 实现检查提交逻辑
3. 实现异常报告功能
4. 支持命令行交互

### 任务 7：教师端图片上传工具

**文件清单：**
- `teacher-server/src/storage.rs`（新建）

**实现步骤：**
1. 实现文件存储逻辑（保存到本地目录）
2. 实现文件类型验证（仅允许图片）
3. 实现文件大小限制（最大 2MB）
4. 实现图片压缩（可选）

### 任务 8：学生端登录模块

**文件清单：**
- `student-agent/agent-core/src/student_auth.rs`（新建）
- `student-agent/agent-core/src/config.rs`（更新）

**实现步骤：**
1. 扩展 Config 结构（学生账号字段）
2. 实现登录 API 调用
3. 实现登录状态持久化
4. 支持自动登录

### 任务 9：教师端 Web 签到页面

**文件清单：**
- `teacher-web/src/views/Attendance.vue`（新建）
- `teacher-web/src/api/attendance.ts`（新建）
- `teacher-web/src/components/attendance/*`（新建组件）

**实现步骤：**
1. 创建 API 封装
2. 创建统计卡片组件
3. 创建签到明细表
4. 创建补签弹窗
5. 集成路由

### 任务 10：教师端 Web 告警处理页面

**文件清单：**
- `teacher-web/src/views/Alerts.vue`（新建）
- `teacher-web/src/api/alerts.ts`（新建）
- `teacher-web/src/components/alerts/*`（新建组件）

**实现步骤：**
1. 创建告警列表页面
2. 创建告警详情弹窗
3. 创建图片上传组件（文件选择）
4. 创建图片预览组件
5. 创建告警处理表单（状态变更、备注）
6. 集成路由

### 任务 11：联调验证

**验证步骤：**
1. 启动教师端服务和 Web
2. 启动学生端 Agent
3. 验证签到流程
4. 验证检查流程
5. 验证拍照上传
6. 验证学生登录
7. 生成 P6 联调验证报告

## 五、验收标准

### 功能验收

- [ ] 学生端可成功签到
- [ ] 教师端可查询签到统计
- [ ] 教师可执行补签操作
- [ ] 学生端可提交检查记录
- [ ] 学生端可报告异常（卫生/设备）
- [ ] 教师端收到告警通知
- [ ] 教师端可查看告警列表
- [ ] 告警处理界面可上传图片
- [ ] 支持从本地文件选择上传
- [ ] 支持教师机摄像头拍照上传
- [ ] 图片与告警记录关联
- [ ] 图片可预览
- [ ] 学生端可登录/退出登录
- [ ] 登录状态可持久化（重启后保留）
- [ ] 支持自动登录（7 天有效期）

### 性能验收

- [ ] 签到响应时间 < 1 秒
- [ ] 照片上传支持批量（10 张以内）
- [ ] 照片自动压缩（< 500KB/张）
- [ ] 签到统计查询 < 2 秒

### 代码质量验收

- [ ] 所有新增代码通过 `cargo clippy`
- [ ] 所有测试通过 `cargo test`
- [ ] 关键函数有单元测试
- [ ] 错误处理完整
- [ ] 日志记录清晰

## 六、风险与应对

### 风险 1：图片存储空间占用

**应对：**
- 默认存储 90 天图片，超期自动清理
- 图片自动压缩（JPEG 80% 质量）
- 单张图片限制最大 2MB
- 支持配置对象存储（后续扩展）

### 风险 2：签到并发压力

**应对：**
- 数据库添加索引（student_id, check_in_time）
- 批量签到请求合并处理
- 支持异步写入（消息队列后续扩展）

## 七、下一步

1. 本执行基线文档经用户确认后生效
2. 按任务 1→11 的顺序依次实现
3. 每个任务完成后进行单元验证
4. 所有任务完成后进行端到端联调
5. 生成 P6 联调验证报告并提交

---

**文档版本：** v1.0  
**生成时间：** 2026-06-08  
**状态：** 待用户确认
