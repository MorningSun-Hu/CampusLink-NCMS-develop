# CampusLink-NCMS 架构概览

## 教师端

- `teacher-server/` 提供 REST API、WebSocket 会话、设备管理、策略管理、日志管理
- `teacher-web/` 提供教师管理后台，负责展示、配置和控制

## 学生端

- `agent-core/`（二进制 `student`）：注册、心跳、模式、拉起准入/锁屏
- `campus-checkin/`：开放/授课强制签到准入（身份确认 + 环境设备检查）
- `campus-guard/`：守护 `student.exe`，异常退出自动拉起
- `campus-lock/`：全屏锁定和输入限制界面；解锁后由主进程恢复锁定前模式

## 通信

- REST API 用于教师端后台管理
- WebSocket 用于教师端服务与学生端设备实时通信
- REST 管理接口与公开上报接口分层（JWT 保护管理面；设备上报按 device_id 校验）
- WebSocket 消息支持 JSON 与 Protobuf；AES256-GCM 加密消息体
- 教师端二进制 `teacher`，内嵌 `static/`，默认监听 8080
