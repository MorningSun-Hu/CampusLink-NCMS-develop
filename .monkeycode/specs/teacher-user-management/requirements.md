# Requirements Document

## Introduction

教师端增加用户管理能力：内置 `admin` 账号负责创建、修改、删除教师账号；教师账号登录后使用教师端现有业务功能。`admin` 账号保持系统内唯一内置管理员身份，其密码仅由 `admin` 本人修改。

## Glossary

- **内置管理员**：`admin_users` 表中 `username` 为 `admin` 的账号，角色为 `admin`
- **教师账号**：`admin_users` 表中角色为 `teacher` 的账号
- **教师端**：`teacher-web` 管理界面及其受 JWT 保护的后端接口
- **用户管理页**：教师端中用于列出、新增、编辑、删除教师账号的页面
- **当前登录用户**：JWT 中 `sub` 对应的 `admin_users` 记录

## Confirmed Decisions

经用户确认的实现口径：

1. 教师账号登录后可使用教师端除用户管理外的全部现有业务功能；用户管理页与用户管理接口仅对内置管理员开放。
2. 内置管理员创建教师账号时指定初始密码，并可重置教师密码；教师账号可修改本人密码；内置管理员密码仅由内置管理员本人修改。
3. 内置管理员可停用或删除教师账号；内置管理员账号保持启用且不可被删除、不可被停用、用户名不可被修改。

## Requirements

### Requirement 1

**User Story:** AS 内置管理员, I want 使用现有登录入口进入教师端, so that 继续管理教室并管理教师账号

#### Acceptance Criteria

1. WHEN 调用教师端登录接口且用户名为 `admin`、密码正确、账号状态为 `active`，THE 系统 SHALL 签发 JWT，JWT 的 `role` 字段值为 `admin`
2. WHEN JWT 的 `role` 为 `admin`，THE 教师端 SHALL 在侧栏展示「用户管理」入口
3. WHEN 内置管理员访问用户管理页，THE 系统 SHALL 列出 `admin_users` 中的全部账号，字段至少包含用户名、显示名、角色、状态、最近登录时间

### Requirement 2

**User Story:** AS 内置管理员, I want 新增教师账号, so that 其他教师可以登录使用教师端

#### Acceptance Criteria

1. WHEN 内置管理员提交新增教师请求且用户名、初始密码、显示名均非空，THE 系统 SHALL 在 `admin_users` 中创建一条记录，`role` 为 `teacher`，`status` 为 `active`
2. WHEN 新增教师的用户名与已有账号重复，THE 系统 SHALL 拒绝创建并返回可理解的错误信息
3. WHEN 新增教师请求的用户名为 `admin`，THE 系统 SHALL 拒绝创建
4. WHEN 教师账号使用正确用户名与密码登录，THE 系统 SHALL 签发 JWT，JWT 的 `role` 字段值为 `teacher`

### Requirement 3

**User Story:** AS 内置管理员, I want 修改和删除教师账号, so that 账号信息保持准确、离职教师无法继续登录

#### Acceptance Criteria

1. WHEN 内置管理员修改教师账号的显示名或状态，THE 系统 SHALL 更新对应 `admin_users` 记录
2. WHEN 内置管理员将教师账号状态改为非 `active`，THE 系统 SHALL 在该账号后续登录时拒绝签发令牌
3. WHEN 内置管理员删除教师账号，THE 系统 SHALL 从 `admin_users` 中移除该记录
4. WHEN 删除或停用目标的用户名为 `admin`，THE 系统 SHALL 保持该记录不变并返回错误信息
5. WHEN 修改请求试图把任意账号的用户名改为 `admin`，或修改内置管理员的用户名，THE 系统 SHALL 拒绝该请求

### Requirement 4

**User Story:** AS 内置管理员, I want 仅由本人修改 admin 密码, so that 内置管理员凭证不被其他账号改写

#### Acceptance Criteria

1. WHEN 当前登录用户为内置管理员并提交修改本人密码请求，且当前密码校验通过、新密码长度不少于 4 位，THE 系统 SHALL 更新内置管理员的密码哈希
2. WHEN 当前登录用户为教师账号并请求修改内置管理员密码，THE 系统 SHALL 拒绝该请求
3. WHEN 内置管理员提交重置教师账号密码请求且新密码长度不少于 4 位，THE 系统 SHALL 更新该教师账号的密码哈希
4. WHEN 当前登录用户为教师账号并提交修改本人密码请求，且当前密码校验通过、新密码长度不少于 4 位，THE 系统 SHALL 更新该教师账号的密码哈希

### Requirement 5

**User Story:** AS 教师, I want 用教师账号登录并使用教师端业务功能, so that 日常管理不必共用 admin 密码

#### Acceptance Criteria

1. WHEN JWT 的 `role` 为 `teacher`，THE 教师端 SHALL 允许访问除用户管理页以外的现有业务页面
2. WHEN JWT 的 `role` 为 `teacher`，THE 教师端 SHALL 隐藏「用户管理」入口
3. WHEN JWT 的 `role` 为 `teacher` 的请求访问用户管理相关接口，THE 系统 SHALL 返回 403 并保持用户数据不变
4. WHEN JWT 的 `role` 为 `admin` 或 `teacher`，THE 系统 SHALL 允许访问教师端现有业务接口（设备、班级、学生、签到、使用记录、告警、硬件、日志、设置、维修、白名单、网络认证）
5. WHEN 教师端已登录，THE 页头 SHALL 显示当前登录用户的显示名；显示名为空时显示用户名

### Requirement 6

**User Story:** AS 系统, I want 用户管理操作留下可核对的结果, so that 账号变更可被测试验证

#### Acceptance Criteria

1. WHEN 列出用户，THE 响应 SHALL 省略密码哈希
2. WHEN 创建、修改、删除、重置密码成功，THE 系统 SHALL 返回统一 `ApiResponse` 成功结构
3. WHEN 请求缺少有效 JWT，THE 用户管理接口 SHALL 返回 401
