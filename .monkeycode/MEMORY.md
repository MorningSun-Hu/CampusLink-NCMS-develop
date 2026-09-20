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
  - `cargo test` 不会更新运行用二进制，修改 main.rs 后需先 `cargo build` 再启动服务验证
  - 教师端二进制名为 `teacher`（`cargo build --bin teacher` / `cargo test --bin teacher`）；学生端主进程二进制名为 `student`（`cargo build -p agent-core`）
  - domain 层已建立内存 SQLite 单测体系：`domain/test_support.rs` 提供 `setup_pool()`（跑全量迁移）与设备/学生注册辅助函数，运行 `cargo test --all-targets` 即可（23 个测试）
  - 学生端上报接口已改为公开路由+device_id 校验（check-in/inspection-submit/hardware-snapshot + policies/process-guard/sync），教师端查询类接口仍受 JWT 保护
  - Windows 交叉编译：需 `apt-get install gcc-mingw-w64-x86-64` + `rustup target add x86_64-pc-windows-gnu`，用 `CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER=x86_64-w64-mingw32-gcc cargo build --release --target x86_64-pc-windows-gnu`（勿在项目写死 .cargo/config.toml，避免污染 Linux 构建）
  - teacher-server Windows 编译约 12 分钟、student-agent（含 eframe/egui 的 campus-lock）约 11 分钟；产物均静态链接只依赖 Windows 系统 DLL
  - Windows 发布包位置 `dist/windows-release/`：教师端双击 `teacher.exe`，学生端双击 `student.exe`；exe 启动时自动切到自身目录并创建 data/logs/config。zip 包 `dist/CampusLink-NCMS-windows-v1.1.zip`
  - student-agent 配置加密存于 `config/config.enc`，默认连接 localhost:8080 或 UDP 9999 广播自动发现教师端；锁屏密码默认 admin123
  - Windows bat 脚本必须用 CRLF 换行（cmd 不认 LF，块结构 `( ) else ( )` 会报"此时不应有..."），且避免块结构改用 goto 跳转
  - 教师端 UDP discovery 必须返回本机局域网 IP 而非 0.0.0.0（0.0.0.0 无法作为连接目标，Windows 报 os error 10049）；已改用 if-addrs 枚举非回环 IPv4 接口
  - 注意：`local-ip-address` 0.1 不支持 Windows（仅 unix 且依赖 ipconfig 命令），交叉编译项目禁用；获取本机 IP 用 `if-addrs` 0.15
  - 学生端已把 URL 含 0.0.0.0 视为未配置重新 discovery，discovery 失败时回退 localhost 保存

[磁盘与预览（Agent 实测发现）]
- Date: 2026-09-19
- Context: 继续班级座位/学生导入特性时，teacher-server 编译因磁盘写满失败
- Category: Build & Compilation / Environment Configuration
- Instructions:
  - 根分区约 20G。`student-agent/target` 与 `teacher-server/target` 合计可超过 14G，cargo 会报 `No space left on device`
  - 只编教师端时，可在 `CampusLink-NCMS/student-agent` 执行 `cargo clean` 释放约 8–9G
  - 教师端预览：后端 `cargo run --bin teacher` 监听 8080，前端 `npm run dev` 监听 5173 并反代 `/api`；默认管理员 `admin` / `admin123`

[本地端到端冒烟测试方法（Agent 实测发现）]
- Date: 2026-09-15
- Context: 验证班级/座位/签到/改密/模式切换特性时发现
- Category: Troubleshooting & Debugging / Build & Compilation
- Instructions:
  - 教师端 `teacher` 启动时 `set_current_dir(exe_dir)`，并自动创建 `data/` `logs/` `config/`，缺失时写入默认 `config/config.toml`。冒烟测试把 `teacher` 可执行文件、`static/` 放到同一临时目录即可
  - 冒烟测试用 SQLite 独立库文件（`sqlite:data/xxx.db`），每次换新库名即可获得干净数据，避免删除文件
  - 公开路由（无需 JWT）：`/api/auth/login`、`/api/auth/student-login`、`/api/auth/student-set-password`、`/api/devices/register`、`/api/attendance/check-in`、`/api/attendance/context`；班级/学生/设备等管理接口需 Bearer JWT
  - 学生改密接口 `/api/auth/student-set-password` 请求体为 snake_case（`student_id`/`old_password`/`new_password`），已用 serde alias 兼容 camelCase；学生登录 `/api/auth/student-login` 仅按学号精确匹配，初始密码默认 `123456`
  - 签到 `requiresCheckin` 语义：`open`/`teaching` 为 true，`exam`/`locked` 为 false；授课模式下学生座位号必须与设备座位号一致
  - 设备重复注册（同 `device_code`）必须返回已存在记录的主键 id，不能返回新生成的 UUID，否则按设备 id 的后续操作（归班/座位/签到）会全部失效（已修复 + 回归测试 `re_register_same_device_code_keeps_stable_id`）

[apply_patch 标记格式]
- Date: 2026-09-19
- Context: 用户要求定位 missing Begin/End markers 并记住，避免再犯
- Category: Environment Configuration
- Instructions:
  - apply_patch 的 `patchText` 必须以整行 `*** Begin Patch` 开头、整行 `*** End Patch` 结尾
  - 这两行必须精确匹配：行首无空格、行尾无多余 `***`、无引号、无代码围栏包裹
  - 错误写法 `*** Begin Patch ***` / `*** End Patch ***` 会报 `Invalid patch format: missing Begin/End markers`
  - 报错 `Failed to find context` 是定位上下文与文件内容对不上，和 Begin/End 标记无关；先读文件，用 `@@` 后跟文件中真实存在的一行来定位
  - 不要写 unified diff 的行号头（例如 `@@ -64,6 +64,19 @@`），该工具会把这串当成要查找的上下文
  - 新建文件用 `*** Add File: <path>`，后续每行内容必须以 `+` 开头
