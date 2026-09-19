# Requirements Document

## Introduction

教师端学生管理补齐导入模板、带登录态导出，以及新增/导入时班级与座位号必填。座位号必须已在该班级的设备上分配。

## Glossary

- **教师端**：teacher-web + teacher-server
- **班级**：`classes` 表记录
- **座位号**：班级内设备 `student_devices.seat_no` 已分配的编号
- **导入模板**：xlsx 文件，含表头与填写说明

## Requirements

### Requirement 1: 导入模板

**User Story:** AS 教师, I want 下载学生导入模板, so that 按列填写后批量导入。

#### Acceptance Criteria

1. WHEN 教师点击「下载模板」, 教师端 SHALL 使用当前登录凭证请求模板文件并下载 `students-template.xlsx`。
2. WHEN 教师打开模板, 文件 SHALL 包含表头：学号、姓名、班级、座位号、密码。
3. WHEN 学号或密码单元格为空, 系统 SHALL 在导入时分别自动生成学号、使用初始密码 `123456`。

### Requirement 2: 导出携带登录态

**User Story:** AS 教师, I want 导出学生名单时保持已登录状态, so that 导出成功并得到 Excel。

#### Acceptance Criteria

1. WHEN 教师点击「导出 Excel」, 教师端 SHALL 使用当前登录凭证请求 `/api/students/export`。
2. WHEN 导出成功, 教师端 SHALL 下载 xlsx，列包含学号、姓名、班级、座位号、状态、创建时间。
3. IF 登录凭证缺失或失效, 系统 SHALL 提示重新登录。

### Requirement 3: 班级与座位必填

**User Story:** AS 教师, I want 新增、编辑、导入学生时必须填写班级和座位号, so that 每个学生都能对应授课座位。

#### Acceptance Criteria

1. WHEN 教师新增或编辑学生, 教师端 SHALL 将班级、座位号标为必填。
2. WHEN 后端创建或更新学生, 系统 SHALL 在班级或座位号为空时拒绝并返回中文原因。
3. WHEN 教师导入学生, 每一行 SHALL 填写已存在的班级名称与座位号。

### Requirement 4: 座位必须已在班级设备上存在

**User Story:** AS 教师, I want 导入或保存学生时校验座位已分配给班级设备, so that 不会出现无法匹配设备的学生座位。

#### Acceptance Criteria

1. WHEN 保存或导入学生, 系统 SHALL 校验目标班级存在。
2. WHEN 保存或导入学生, 系统 SHALL 校验该班级下至少一台设备的 `seat_no` 与学生座位号一致（去空白后比较）。
3. IF 班级名称不存在或座位号在该班设备中找不到, 系统 SHALL 拒绝该次创建/更新，或拒绝整份导入文件，并返回中文原因。
4. IF 导入文件中任一行校验失败, 系统 SHALL 保持数据库中原有学生记录不变。
