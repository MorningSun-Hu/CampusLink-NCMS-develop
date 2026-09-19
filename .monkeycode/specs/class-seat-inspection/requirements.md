# Requirements Document

## Introduction

班级管理支持批量添加学生机、自动/批量设置座位号、导出导入座位；检查告警页一次检查显示一条记录并展示设备名称。

## Glossary

- **学生机**：已注册的 `student_devices` 记录
- **座位号**：班级内学生机的 `seat_no`，供学生名单匹配
- **检查记录**：学生机一次提交检查形成的汇总行

## Requirements

### Requirement 1: 批量添加学生机并自动编号

**User Story:** AS 教师, I want 批量把学生机加入班级并自动分配座位号, so that 随后能按座位新增或导入学生。

#### Acceptance Criteria

1. WHEN 教师在班级详情点击「添加学生机」, 教师端 SHALL 展示可多选的学生机列表（设备名、IP、在线状态）。
2. WHEN 教师确认添加, 系统 SHALL 将这些学生机归入当前班级。
3. WHEN 添加完成, 系统 SHALL 为尚无座位号的学生机按 1、2、3… 分配未被占用的座位号。

### Requirement 2: 批量修改与导出导入座位号

**User Story:** AS 教师, I want 批量改座位号或导出名单改完再导入, so that 不必逐台反复保存失败。

#### Acceptance Criteria

1. WHEN 教师在班级学生机表中填写座位号并保存, 系统 SHALL 将座位号写入对应学生机，成功后刷新列表。
2. WHEN 教师点击「顺序编号」, 系统 SHALL 按当前列表顺序把学生机座位号重排为 1、2、3…。
3. WHEN 教师导出座位表, 教师端 SHALL 下载含设备名、IP、座位号的 xlsx。
4. WHEN 教师导入座位表, 系统 SHALL 按设备 ID 更新座位号；IF 座位号重复或设备不属于该班, 系统 SHALL 拒绝整份文件并返回中文原因。

### Requirement 3: 班级操作命名

**User Story:** AS 教师, I want 按钮名称与教室场景一致, so that 能直接找到添加学生和添加学生机。

#### Acceptance Criteria

1. WHEN 教师查看班级详情, 原「归入学生」SHALL 显示为「添加学生」。
2. WHEN 教师查看班级详情, 原「归入设备」SHALL 显示为「添加学生机」。

### Requirement 4: 检查记录按次汇总

**User Story:** AS 教师, I want 一次检查只看到一条记录并看到设备名称, so that 和设备使用页一样能定位机器。

#### Acceptance Criteria

1. WHEN 学生机提交一次检查（含多个检查项）, 系统 SHALL 写入一条检查记录。
2. WHEN 教师打开检查记录表, 教师端 SHALL 显示设备名称列。
3. WHEN 任一检查项为异常或缺失, 该条检查记录状态 SHALL 为异常，描述中列出各项结果。
