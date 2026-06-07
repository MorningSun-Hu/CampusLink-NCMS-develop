# CampusLink-NCMS 架构概览

## 教师端

- `teacher-server/` 提供 REST API、WebSocket 会话、设备管理、策略管理、日志管理
- `teacher-web/` 提供教师管理后台，负责展示、配置和控制

## 学生端

- `agent-core/` 负责业务主流程、注册、心跳、签到、检查、网络认证
- `campus-guard/` 负责守护与异常拉起
- `campus-lock/` 负责全屏锁定和输入限制界面

## 通信

- REST API 用于教师端后台管理
- WebSocket 用于教师端服务与学生端设备实时通信
- Protobuf 用于消息序列化
- AES256-GCM 用于消息体加密
