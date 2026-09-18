# 系统架构

完整说明在当前工作区 `/CampusLink-NCMS/docs/architecture-overview.md` 与 `/CampusLink-NCMS/docs/p12-stage-summary.md`。

## 运行时拓扑

- 教师机：`teacher.exe` 提供 REST、WebSocket、内嵌 Web（8080），SQLite 在 `data/`
- 学生机：`student.exe` 主进程；`campus-checkin.exe` 开放/授课准入；`campus-lock.exe` 考试/锁定；`campus-guard.exe` 只守护 `student.exe`

## 模式

| 模式 | 学生机行为 |
|------|------------|
| open | 强制签到准入，使用记录写入 `device_usage_records` |
| teaching | 强制签到准入，考勤写入 `attendance_records`；座位须匹配 |
| exam / locked | 全屏锁定，不要求签到 |

教师端「锁屏」为 overlay，解锁恢复 `mode_before_lock`。
