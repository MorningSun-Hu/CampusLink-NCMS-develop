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
