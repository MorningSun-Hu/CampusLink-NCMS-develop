# 需求实施计划

- [ ] 1. 数据库迁移与基础模型扩展
  - [x] 1.1 新增 `classes` 表迁移（0016_create_classes.sql），含名称唯一约束
  - [x] 1.2 新增字段迁移（0017_add_class_and_signin_fields.sql）：`students` 增 `class_id`/`password_set`；`student_devices` 增 `class_id`/`seat_no`/`pending_checkin`；`attendance_records` 增 `seat_no`
  - [x] 1.3 更新领域结构体与响应类型（`StudentRow`/`DeviceRow`/`AttendanceRow` 及 Response），透出新字段
  - [ ]* 1.4 为迁移与模型扩展编写单元测试

- [ ] 2. 班级领域与座位号管理（Requirement 6、7）
  - [x] 2.1 实现班级 CRUD 与名称唯一校验
  - [x] 2.2 实现学生归班（一人至多一班）与班级学生查询
  - [x] 2.3 实现按班级查询关联设备
  - [x] 2.4 实现座位号批量设置、按设备 IP 自动分配，并保证班级内唯一
  - [ ]* 2.5 编写班级与座位号单元测试及唯一性属性测试

- [ ] 3. 学生与签到密码（Requirement 8、9）
  - [x] 3.1 调整导入逻辑：自动生成学号、记录学生座位号
  - [x] 3.2 实现初始密码发放与 `password_set` 语义（0=初始，1=已自定义）
  - [x] 3.3 改造 `student_login`：支持按学号或姓名登录，返回是否需修改密码
  - [x] 3.4 实现首次授课签到修改密码接口
  - [x] 3.5 实现教师重置与按班级批量重置密码
  - [ ]* 3.6 编写密码流程单元测试与属性测试

- [ ] 4. 签到逻辑增强（Requirement 2、7、8）
  - [x] 4.1 签到记录返回并展示学生姓名/学号/座位号
  - [x] 4.2 授课模式校验学生座位号与设备座位号一致
  - [x] 4.3 签到写入座位号快照
  - [x] 4.4 新增签到上下文接口（返回当前模式/班级，用于「无需签到」提示）
  - [ ]* 4.5 编写签到单元测试与座位匹配属性测试

- [ ] 5. 时间字段时区统一（Requirement 1）
  - [x] 5.1 设备、签到、学生时间字段统一以 RFC3339 UTC 返回
  - [ ]* 5.2 编写时间序列化属性测试

- [ ] 6. 模式切换与签到触发（Requirement 3、4、5、6）
  - [x] 6.1 授课模式切换前置校验：所选班级学生非空
  - [x] 6.2 实现按班级批量切换模式
  - [x] 6.3 下发 `checkin_trigger`，离线设备写入 `pending_checkin`
  - [x] 6.4 心跳响应补发 pending 签到并清零标志
  - [ ]* 6.5 编写模式切换与离线补发测试

- [ ] 7. 后端接口与路由
  - [x] 7.1 新增班级、座位、密码重置 handlers
  - [x] 7.2 注册公开路由（student-login/student-set-password/check-in/attendance-context）与 JWT 保护路由
  - [x] 7.3 适配 attendance/student/device handlers 新字段与校验
  - [ ]* 7.4 编写接口集成测试

- [x] 8. 检查点 - 确保后端编译与测试通过,如有疑问请询问用户

- [ ] 9. 设备端实施（Requirement 4、7、8）
  - [x] 9.1 `command_handler` 处理 `checkin_trigger`，模式切换后按模式触发签到
  - [x] 9.2 `attendance` 支持命令重复触发并携带设备座位号
  - [x] 9.3 `campus-checkin` 授课模式初始密码校验与强制改密流程
  - [ ]* 9.4 编写设备端单元测试

- [x] 10. 教师端前端实施
  - [x] 10.1 新增时间工具 `utils/time.ts`，修正 Monitor/Devices 时间与状态判定
  - [x] 10.2 Attendance 增姓名/学号/座位号列，并按上下文显示「当前模式不需要签到」
  - [x] 10.3 Devices 授课模式切换增加班级选择
  - [x] 10.4 新增 Classes 班级管理页（归班、座位批量/按 IP、批量重置密码）
  - [x] 10.5 Students 增加班级、密码状态与重置
  - [x] 10.6 新增 `api/classes.ts` 与相关类型
  - [ ]* 10.7 编写前端单元测试

- [x] 11. 检查点 - 确保前端构建与后端联调通过,如有疑问请询问用户

- [x] 12. 打包与产物同步
  - [x] 12.1 构建前端并同步到 `teacher-server/static/`
  - [x] 12.2 交叉编译 Windows teacher-server 与 student-agent
  - [x] 12.3 更新 `dist/windows-release/` 并重建发布 zip
