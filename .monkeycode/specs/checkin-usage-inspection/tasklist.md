# 需求实施计划

- [x] 1. 数据模型与迁移（设计：数据模型 / 使用记录）
  - [x] 1.1 新增迁移 `0018_device_usage_records.sql`
    - 创建 `device_usage_records` 表（device_id/class_id/seat_no/student_id/user_name/mode/start_time/end_time/duration_seconds/inspection_ok/inspection_summary/created_at）
    - 为 `attendance_records` 增加 `usage_record_id TEXT NULL`
  - [x] 1.2 实现 `domain/usage.rs`
    - 定义 `UsageRow`/`UsageRecord` 响应类型（camelCase，时间 RFC3339）
    - 实现使用会话开启、关闭（写入 end_time 与 duration_seconds）、按条件查询与导出数据
    - 同一设备至多一条未结束会话（正确性属性 3、4）
  - [x] 1.3 在 `domain/mod.rs` 注册模块并复用 `time_util`
  - [ ]* 1.4 编写使用记录单元测试
    - 覆盖会话开启、正常关闭、重复关闭不覆盖 end_time、时长为负时归零
    - 覆盖同一设备并发开启时仅保留一条未结束会话

- [x] 2. 授课模式准入校验与阻断（需求 1）
  - [x] 2.1 `domain/device.rs` 的 `switch_device_mode` 支持 `class_id` 准入
    - 未提供 class_id 时对授课模式返回错误（需求 1.1）
    - 班级学生数为 0 时拒绝切换且不修改模式（需求 1.2、正确性属性 7）
    - 设备座位号与班级学生座位号匹配时进入授课模式，否则返回锁定模式目标（需求 1.3、1.4）
  - [x] 2.2 `domain/class.rs` 的 `switch_class_mode` 返回分组结果
    - 输出 `teaching_device_ids`、`locked_device_ids` 与各自数量（需求 1.5、1.6）
  - [x] 2.3 调整 `api/handlers.rs` 与 `api/class_handlers.rs`
    - `ModeSwitchRequest` 增加可选 `class_id`
    - 授课模式校验失败返回 400 与中文提示，成功时按分组下发 mode_switch 指令（需求 1.7）
  - [ ]* 2.4 编写准入校验单元测试
    - 无学生拒绝、座位匹配进入授课、不匹配进入锁定、非授课模式切换不受影响（需求 1.8）

- [x] 3. 考勤化签到与看板（需求 2、3）
  - [x] 3.1 拆分 `domain/attendance.rs` 的 `check_in`
    - 授课模式写入 `attendance_records` 并保留座位校验（需求 2.3）
    - 开放模式写入 `device_usage_records`，不再写入考勤（需求 3.1、3.2、正确性属性 5）
    - 开放模式使用人姓名写入使用记录 `user_name`（需求 3.4）
  - [x] 3.2 考勤查询过滤（需求 2.1、2.2）
    - `list_attendance` 支持 `class_id`、`start_date`、`end_date`、`student_keyword`、`status`
    - 过滤层排除开放模式与历史 `open-checkin:` 记录（需求 3.3）
  - [x] 3.3 实现考勤看板 `attendance_board`
    - 按班级与日期计算应出勤、已签到、缺勤、出勤率与缺勤名单（需求 2.4、2.5、正确性属性 1、2）
  - [x] 3.4 导出支持筛选条件（需求 2.6）
    - `export_attendance_handler` 复用查询过滤，导出字段含姓名、学号、班级、座位号、设备、签到时间、状态
  - [ ]* 3.5 编写考勤与看板单元测试
    - 覆盖出勤/缺勤互补、出勤率计算、开放签到不影响考勤、无匹配记录空结果（需求 2.7）

- [x] 4. 签到内嵌环境与设备检查（需求 5）
  - [x] 4.1 `check_in` 请求支持 `inspection_items` 与 `is_abnormal`
    - 存在检查项时写入 `inspection_records`，异常项生成告警（需求 5.3、5.4）
  - [x] 4.2 使用记录保存检查结果
    - 写入 `inspection_ok` 与 `inspection_summary`（需求 5.3、6.2）
  - [x] 4.3 检查异常处理策略
    - 异常时记录告警但仍返回签到成功（需求 5.4、已确认决策 3）
  - [ ]* 4.4 编写检查写入与告警单元测试
    - 覆盖正常项不告警、异常项生成告警、检查失败仍完成签到

- [x] 5. 设备使用记录接口（需求 6）
  - [x] 5.1 实现 `GET /api/usage` 查询
    - 支持 `device_id`、`class_id`、`student_keyword`、`start_date`、`end_date` 过滤（需求 6.5）
    - 返回使用人、开始/结束时间、时长与检查结果摘要（需求 6.6）
  - [x] 5.2 实现 `POST /api/usage/:id/end` 手动结束会话（需求 6.4）
  - [x] 5.3 实现 `GET /api/usage/export` 导出（需求 6.7）
  - [x] 5.4 在 `api/mod.rs` 注册使用记录路由
  - [ ]* 5.5 编写使用记录接口单元测试
    - 覆盖过滤条件、手动结束、导出字段、空结果（需求 6.8）

- [x] 6. 检查点 - 确保后端编译与测试通过,如有疑问请询问用户

- [x] 7. 设备端强制签到准入（需求 3、4、5）
  - [x] 7.1 `campus-checkin` 窗口改为全屏准入
    - 设置为全屏、无边框、置顶（需求 4.1）
    - 新增 `keyhook.rs` 并屏蔽 Windows 键、Alt+Tab、Alt+F4、Ctrl+Esc、任务管理器与全屏切换快捷键（需求 4.2）
  - [x] 7.2 `campus-checkin` 分阶段流程
    - 阶段一身份确认（open 填姓名；teaching 学号或姓名加密码，首次签到强制改密）（需求 4.1、4.5）
    - 阶段二环境与设备检查展示与提交（需求 5.1、5.2、5.6）
    - 阶段三提交签到后退出并返回完成退出码（需求 4.4）
  - [x] 7.3 签到请求携带检查项
    - 使用 `hardware.rs` 与 `inspection.rs` 采集项，提交 `inspection_items` 与 `is_abnormal`（需求 5.3）
  - [x] 7.4 `agent-core/src/mode.rs` 与 `attendance.rs` 拉起并监控准入
    - `open` 与 `teaching` 模式进入准入状态，未完成不得进入桌面（需求 4.1、4.5）
    - 准入界面异常退出时重新拉起（需求 4.3）
    - 教师超级密码解锁并记录（需求 4.6）
    - `exam`/`locked` 保持现有锁定策略（需求 4.7）
  - [x] 7.5 使用会话结束联动
    - 准入界面关闭、模式切离 `open`/`teaching`、新签到发生时通知服务端关闭会话（需求 6.4、已确认决策 3）
  - [ ]* 7.6 编写设备端单元测试
    - 覆盖检查项构建与异常判定、快捷键拦截判定、准入退出码语义

- [x] 8. 锁屏界面中文化与美化（需求 7）
  - [x] 8.1 `campus-lock/src/main.rs` 文案改为简体中文（需求 7.1）
  - [x] 8.2 布局分区优化
    - 按标题区、说明区、密码输入区、操作区、状态区分区（需求 7.3）
    - 状态区展示剩余尝试次数、失败原因与成功提示（需求 7.4、7.5、7.6）
    - 保留深色背景风格（需求 7.2）

- [x] 9. 教师端考勤管理与看板（需求 2）
  - [x] 9.1 扩展 `api/attendance.ts` 与 `api/types.ts`
    - 新增过滤参数、考勤看板与导出接口（需求 2.2、2.6）
  - [x] 9.2 改造 `views/Attendance.vue`
    - 新增班级、日期范围、学生关键字与状态筛选（需求 2.2）
    - 展示学号、姓名、班级、座位号、设备、签到时间与状态（需求 2.3）
    - 新增看板卡片与缺勤名单（需求 2.4、2.5、2.7）
    - 新增导出按钮并传递当前筛选条件（需求 2.6）
    - 移除开放模式签到展示（需求 2.1）

- [x] 10. 教师端设备使用记录模块（需求 6）
  - [x] 10.1 新增 `api/usage.ts` 与类型（需求 6.5、6.7）
  - [x] 10.2 新增 `views/Usage.vue`
    - 筛选区（设备、班级、学生、日期范围）（需求 6.5）
    - 列表展示使用人、开始/结束时间、时长与检查结果摘要（需求 6.6）
    - 手动结束会话与导出（需求 6.4、6.7）
    - 空状态提示（需求 6.8）
  - [x] 10.3 在 `router/index.ts` 与 `components/Layout.vue` 注册「设备使用」菜单与路由

- [x] 11. 教师端授课模式切换班级选择（需求 1）
  - [x] 11.1 扩展 `api/devices.ts` 切换接口支持 `classId`
  - [x] 11.2 改造 `views/Devices.vue` 模式切换弹窗
    - 授课模式要求选择班级后方可提交（需求 1.1）
    - 展示进入授课模式与进入锁定模式的设备数量（需求 1.6）
    - 切换被拒绝时展示原因并保持原模式（需求 1.7）

- [x] 12. 检查点 - 确保前端构建与前后端联调通过,如有疑问请询问用户

- [x] 13. 打包与产物同步
  - [x] 13.1 构建前端并同步到 `teacher-server/static/`
  - [x] 13.2 交叉编译 Windows teacher-server 与 student-agent
  - [x] 13.3 更新 `dist/windows-release/` 并重建发布 zip
