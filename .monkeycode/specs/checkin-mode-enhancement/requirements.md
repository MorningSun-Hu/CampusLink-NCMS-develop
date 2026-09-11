# Requirements Document

## Introduction

本次需求围绕 CampusLink-NCMS 的「签到」与「模式切换」两条主线，解决教师在监控、签到、模式管理使用中暴露的 5 类问题，并补齐班级、座位号、签到密码等基础能力。目标受众为授课教师（教师端）与教室设备终端（学生端 Agent）。

涉及范围：`teacher-server`（后端）、`teacher-web`（教师端前端）、`student-agent`（设备端，含 agent-core 与 campus-checkin）。

## Glossary

- **教师端**：运行于教师电脑的 Web 管理界面（teacher-web + teacher-server）。
- **设备终端**：安装于学生机房的 `agent-core` / `campus-checkin`。
- **设备**：`student_devices` 表中一台已注册的学生机，具有 `device_code`、`ip_address`、`current_mode`。
- **学生**：`students` 表中的一条记录，具有 `student_no`、`name`、`seat_no`、`password_hash`。
- **班级**：一组学生的集合，本次新增的实体。
- **座位号**：标识学生在授课教室中座位位置的编号，本次要求与学生在授课模式下强绑定。
- **模式**：设备当前状态，取值 `open`（开放）、`teaching`（授课）、`exam`（考试）、`locked`（锁屏）。
- **需要签到的模式**：进入该模式后要求学生在设备上完成签到的模式。
- **签到密码**：学生在设备上完成签到时使用的口令，区别于教师端登录密码与锁屏超级密码。
- **监控总览**：教师端实时展示在线设备与状态的页面（`Monitor.vue`）。
- **签到管理**：教师端展示与操作签到记录的页面（`Attendance.vue`）。

## Requirements

### Requirement 1: 时间戳时区一致性

**User Story:** AS 教师, I want 监控总览显示的心跳时间与状态判定准确, so that 我能据此判断设备是否真正在线。

#### Acceptance Criteria

1. WHEN 后端存储或返回任一设备时间字段（如 `last_seen_at`、`check_in_time`、`created_at`）, 系统 SHALL 使用带明确时区标识的 UTC 时间格式（ISO 8601，形如 `2026-09-11T11:10:40Z`）序列化。
2. WHEN 教师端展示任一后端时间字段, 系统 SHALL 按浏览器所在本地时区转换为本地时间后展示。
3. WHEN 教师端计算设备「在线」或「延迟」状态, 系统 SHALL 使用同一时区基准下的「当前时间」与「最后心跳时间」相减，得到的心跳间隔阈值 SHALL 为 30 秒。
4. IF 设备最后心跳时间与当前时间的时差不超过 30 秒, 系统 SHALL 将状态显示为「在线」。
5. IF 设备最后心跳时间与当前时间的时差超过 30 秒或最后心跳时间为空, 系统 SHALL 将状态显示为「延迟」。

### Requirement 2: 签到记录展示签到者姓名

**User Story:** AS 教师, I want 在签到管理中直接看到签到者姓名, so that 我能快速核对出勤人员。

#### Acceptance Criteria

1. WHEN 签到记录关联到学生, 教师端签到管理 SHALL 显示该学生的姓名。
2. WHEN 签到记录为学生关联为空的开放模式签到, 教师端签到管理 SHALL 显示签到备注中记录的使用者姓名。
3. WHEN 教师端展示签到记录, 系统 SHALL 同时展示学生姓名与学生学号两列。

### Requirement 3: 无需签到模式的提示

**User Story:** AS 教师, I want 在当前模式不需要签到时看到明确提示, so that 我不会误以为签到功能故障。

签到模式划分：`open`（开放，宽松签到）与 `teaching`（授课，严格签到）属于需要签到的模式；`exam`（考试）与 `locked`（锁屏）属于不需要签到的模式。

#### Acceptance Criteria

1. WHEN 教师端签到管理页面加载, 系统 SHALL 获取当前生效模式。
2. IF 当前模式为 `exam` 或 `locked`, 系统 SHALL 在签到管理页面显示文案「当前模式不需要签到」。
3. WHILE 当前模式为 `exam` 或 `locked`, 系统 SHALL 隐藏面向学生的签到入口与触发操作。
4. WHEN 当前模式为 `open` 或 `teaching`, 系统 SHALL 在签到管理页面展示签到操作入口。

### Requirement 4: 模式切换触发签到

**User Story:** AS 教师, I want 切换到需要签到的模式时设备立即发起签到, so that 学生无需等待或手动触发。

#### Acceptance Criteria

1. WHEN 教师端将设备切换至需要签到的模式, 系统 SHALL 在模式切换命令下发后立即向该设备下发一次签到触发指令。
2. WHEN 设备终端收到签到触发指令, 系统 SHALL 打开签到界面（`campus-checkin`）。
3. IF 目标设备处于离线状态, 系统 SHALL 记录该设备待触发签到，并在设备恢复在线时补发签到触发指令。
4. IF 目标模式属于需要签到的模式且签到未完成, 系统 SHALL 允许学生多次尝试签到直至成功或模式再次切换。
5. WHEN 设备终端在启动时处于需要签到的模式, 系统 SHALL 继续执行启动即签到流程。

### Requirement 5: 授课模式学生信息前置校验

**User Story:** AS 教师, I want 学生信息不完整时禁止进入授课模式, so that 授课模式不会因缺少学生数据而失效。

#### Acceptance Criteria

1. WHEN 教师端请求切换至授课模式, 系统 SHALL 校验所选范围内是否存在学生记录。
2. IF 所选班级不存在任何学生记录, 系统 SHALL 拒绝进入授课模式并提示教师先补充学生信息。
3. IF 所选班级存在至少一名学生记录, 系统 SHALL 允许切换至授课模式。

### Requirement 6: 班级管理与授课模式班级选择

**User Story:** AS 教师, I want 在进入授课模式时选择班级, so that 授课模式只作用于该班级的学生与设备。

#### Acceptance Criteria

1. 系统 SHALL 提供班级实体，并支持为班级设置名称。
2. 系统 SHALL 支持将学生归属于班级，且每名学生 SHALL 至多归属于一个班级。
3. WHEN 教师端发起切换至授课模式, 系统 SHALL 要求教师选择一个班级作为本次授课范围。
4. WHEN 教师端切换至授课模式并选定班级, 系统 SHALL 仅将该模式作用于所选班级关联的设备。
5. IF 教师未选择班级, 系统 SHALL 阻止提交授课模式切换请求。

### Requirement 7: 座位号绑定与批量管理

**User Story:** AS 教师, I want 在授课模式下将学生与座位号强绑定并批量维护, so that 我能按座位快速定位学生。

#### Acceptance Criteria

1. 系统 SHALL 为每台设备保存一个固定座位号，且同一班级内设备的座位号不重复。
2. WHEN 教师导入学生, 系统 SHALL 记录教师为该学生指定的座位号。
3. WHEN 学生在授课模式下签到, 系统 SHALL 校验学生被指定的座位号与其签到设备的座位号一致。
4. IF 学生座位号与签到设备座位号不一致, 系统 SHALL 拒绝签到并提示座位不匹配。
5. 系统 SHALL 提供按班级批量设置或导入座位号的入口。
6. 系统 SHALL 支持依据学生设备的 IP 地址批量分配座位号。
7. WHILE 设备处于授课模式, 系统 SHALL 在签到与监控展示中呈现其座位号。

### Requirement 8: 首次签到设置密码与校验

**User Story:** AS 教师, I want 学生签到需验证密码且首次签到时自行设置, so that 签到记录不会被冒用。

#### Acceptance Criteria

1. WHEN 教师在教师端导入学生, 系统 SHALL 为每名学生自动生成唯一学号。
2. WHEN 教师在教师端导入学生, 系统 SHALL 为该学生发放一个初始签到密码。
3. WHEN 学生首次于授课模式签到, 系统 SHALL 使用初始签到密码完成校验，并要求学生设置新的签到密码。
4. WHEN 学生已完成签到密码设置并再次签到, 系统 SHALL 校验学生输入的登录标识与签到密码。
5. IF 学生输入的签到密码与已设置密码不一致, 系统 SHALL 拒绝签到并提示密码错误。
6. WHEN 学生在开放模式下签到, 系统 SHALL 仅要求输入使用者姓名并完成签到，无需签到密码。
7. WHEN 学生成功完成签到, 系统 SHALL 记录该学生与学生设备的关联及签到时间。
8. 系统 SHALL 使用加盐哈希方式保存签到密码。

### Requirement 9: 教师端学生密码管理

**User Story:** AS 教师, I want 管理学生的签到密码, so that 我可以处理学生忘记密码或密码异常的情况。

#### Acceptance Criteria

1. WHEN 教师查看学生详情或学生列表, 系统 SHALL 显示该学生是否已完成签到密码设置。
2. WHEN 教师重置学生签到密码, 系统 SHALL 为该学生发放新的初始密码，并将该学生标记为下次签到时需修改密码。
3. WHILE 教师管理学生签到密码, 系统 SHALL 以可追溯的操作日志记录重置动作。
4. 系统 SHALL 支持按班级批量重置学生签到密码。

## 已确认决策

1. **班级归属与模式作用域**：一名学生至多归属一个班级；授课模式按「选择班级 → 批量作用于该班全部关联设备」的方式切换。
2. **首次签到的学生识别方式**：教师在教师端导入学生，系统自动生成学生学号；教师随导入发放初始签到密码，学生首次签到时使用初始密码校验并被要求设置新密码。
3. **需要签到的模式范围**：`open` 为宽松签到（输入使用者姓名即可）；`teaching` 为严格签到（学生与座位号强绑定，且需验证学生密码）；`exam` 与 `locked` 不需要签到。
4. **座位号强绑定落地**：教师预先为每台学生机器设定唯一座位号，导入学生时指定学生座位号；学生仅允许在与其座位号一致的那台设备上签到。
5. **签到触发范围**：模式切换后向班级内全部设备下发签到触发指令；离线设备记录待触发，恢复在线时补发。

## 需求编号与问题映射

| 问题 | 对应用户需求 |
|------|--------------|
| 问题 1（时间/状态） | Requirement 1 |
| 问题 2（姓名、无需签到提示） | Requirement 2、Requirement 3 |
| 问题 3（切换触发签到、授课前置校验） | Requirement 4、Requirement 5 |
| 问题 4（班级、座位号） | Requirement 6、Requirement 7 |
| 问题 5（签到密码） | Requirement 8、Requirement 9 |
