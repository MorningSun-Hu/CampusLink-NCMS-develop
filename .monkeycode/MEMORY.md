# 用户指令记忆

本文件记录了用户的指令、偏好和教导，用于在未来的交互中提供参考。

## 条目

[开发流程规范]
- Date: 2026-06-07
- Context: CampusLink-NCMS 项目初始化阶段
- Instructions:
  - 每次压缩上下文或任务阶段切换时，先读取 `/workspace/.monkeycode/MEMORY.md`
  - 每次提交代码前先取得用户的明确授权
  - Git 提交身份使用 `huchenyang <hcy_1987@163.com>`
  - 当前阶段先做需求分析、目录结构与文件规划，暂不开始业务编码

[开发文档执行规范]
- Date: 2026-06-07
- Context: CampusLink-NCMS 文档化开发阶段
- Instructions:
  - 后续开发前先阅读 `docs/requirements-analysis.md`
  - 再阅读 `docs/development-task-breakdown.md`
  - 然后阅读 `docs/module-roadmap.md`
  - 每一步开发开始前先生成类似 `docs/p0-execution-plan.md` 的执行基线文档
  - 每个阶段的编码、联调、验收都以对应执行基线文档为准

[构建与测试方法（Agent 实测发现）]
- Date: 2026-08-29
- Context: 对 CampusLink-NCMS 做后台自动构建与测试时发现
- Category: Build & Compilation / Environment Configuration
- Instructions:
  - Rust 工具链未预装，需 `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`（profile minimal），装完后 `export PATH="$HOME/.cargo/bin:$PATH"`
  - teacher-server 编译依赖系统库：`apt-get install -y libssl-dev pkg-config`（openssl-sys 需要）与 `protobuf-compiler`（prost-build 需要）
  - 前端 `teacher-web` 的 vue-tsc 1.8.x 与 typescript 5.9 不兼容，`npm run build`（vue-tsc && vite build）会崩溃；需将 vue-tsc 升至 2.x
  - 前端构建产物需手动同步到 `teacher-server/static/` 目录（`cp -r teacher-web/dist/* teacher-server/static/`），SPA fallback 依赖 `static/index.html`
  - 教师端支持 `DATABASE_URL` 环境变量指定 SQLite 路径，测试时可用临时目录避免污染
  - 后台编译任务需用 background_terminal_create 管理；cargo 在 2 核环境编译 teacher-server 约 5-6 分钟、student-agent 约 20 分钟
  - `cargo test` 不会更新 `target/debug/teacher-server` 二进制，修改 main.rs 后需先 `cargo build` 再启动服务验证
  - domain 层已建立内存 SQLite 单测体系：`domain/test_support.rs` 提供 `setup_pool()`（跑全量迁移）与设备/学生注册辅助函数，运行 `cargo test --all-targets` 即可（23 个测试）
  - 学生端上报接口已改为公开路由+device_id 校验（check-in/inspection-submit/hardware-snapshot + policies/process-guard/sync），教师端查询类接口仍受 JWT 保护
