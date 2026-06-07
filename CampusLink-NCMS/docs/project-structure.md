# CampusLink-NCMS 目录结构设计

```text
CampusLink-NCMS/
├─ README.md
├─ .gitignore
├─ docs/
│  ├─ requirements-analysis.md
│  ├─ project-structure.md
│  ├─ architecture-overview.md
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
│  └─ campus-lock/
│     └─ src/
└─ deploy/
   ├─ windows/
   └─ linux/
```

## 设计原则

- 教师端服务与教师端界面分离
- 学生端拆分为主进程、守护进程、锁定进程
- 协议文件单独管理，便于前后端与 Agent 共享
- 文档先行，编码按模块推进
