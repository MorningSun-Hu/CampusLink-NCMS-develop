# CampusLink-NCMS 目录结构设计

```text
CampusLink-NCMS/
├─ README.md
├─ .gitignore
├─ docs/
│  ├─ README.md
│  ├─ p12-stage-summary.md
│  ├─ development-summary.md
│  ├─ requirements-analysis.md
│  ├─ architecture-overview.md
│  ├─ project-structure.md
│  └─ database-outline.md
├─ proto/
│  └─ README.md
├─ scripts/
│  └─ README.md
├─ teacher-server/
│  ├─ Cargo.toml
│  ├─ config/
│  │  └─ config.example.toml
│  ├─ migrations/
│  └─ src/
│     ├─ main.rs
│     ├─ app/
│     ├─ api/
│     ├─ services/
│     ├─ domain/
│     ├─ infrastructure/
│     └─ realtime/
├─ teacher-web/
│  ├─ package.json
│  ├─ tsconfig.json
│  ├─ vite.config.ts
│  ├─ public/
│  └─ src/
│     ├─ main.ts
│     ├─ App.vue
│     ├─ router/
│     ├─ stores/
│     ├─ views/
│     ├─ components/
│     ├─ api/
│     └─ types/
├─ student-agent/
│  ├─ Cargo.toml
│  ├─ agent-core/
│  │  └─ src/
│  ├─ campus-guard/
│  │  └─ src/
│  ├─ campus-lock/
│  │  └─ src/
│  └─ campus-checkin/
│     └─ src/
└─ deploy/
   ├─ windows/
   └─ linux/
```

## 设计原则

- 教师端服务与教师端界面分离
- 学生端拆为主进程 `student`、守护 `campus-guard`、锁定 `campus-lock`、准入 `campus-checkin`
- 协议文件单独管理，便于前后端与 Agent 共享
- 文档先行，编码按模块推进
