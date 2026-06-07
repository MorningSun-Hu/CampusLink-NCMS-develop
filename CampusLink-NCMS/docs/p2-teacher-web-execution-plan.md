# P2 教师端 Web 基础页面执行基线文档

## 一、目标

本执行基线用于指导教师端 Web 前端实现，完成 P0 阶段定义的基础页面，确保教师可以通过 Web 界面查看设备状态和管理设备。

## 二、本次产出范围

1. 项目初始化与基础配置
2. 登录页面（占位）
3. 仪表盘页面
4. 设备列表页面
5. 监控总览页面
6. API 封装

## 三、技术栈

- Vue 3 + TypeScript
- Vite
- Element Plus
- Pinia
- Vue Router
- Axios

## 四、页面清单

### 1. 登录页 `/login`

- 账号输入框
- 密码输入框
- 登录按钮
- 登录后跳转仪表盘

### 2. 仪表盘 `/dashboard`

- 学生总数卡片
- 已注册设备数卡片
- 在线设备数卡片
- 离线设备数卡片

### 3. 设备列表页 `/devices`

- 设备表格
- 显示字段：设备编号、设备名称、主机名、IP 地址、注册状态、在线状态、最后心跳时间
- 支持在线状态筛选
- 支持刷新按钮

### 4. 监控总览页 `/monitor`

- 在线设备列表
- 显示设备实时状态
- 显示最后心跳时间

## 五、API 封装

- `api/dashboard.ts` - 仪表盘数据接口
- `api/devices.ts` - 设备管理接口
- `api/types.ts` - TypeScript 类型定义

## 六、接口依赖

- `GET /api/health`
- `POST /api/devices/register`
- `POST /api/devices`
- `GET /api/dashboard/overview` (预留)

## 七、目录结构

```
teacher-web/
├─ src/
│  ├─ main.ts
│  ├─ App.vue
│  ├─ router/
│  │  └─ index.ts
│  ├─ stores/
│  │  ├─ device.ts
│  │  └─ dashboard.ts
│  ├─ views/
│  │  ├─ Login.vue
│  │  ├─ Dashboard.vue
│  │  ├─ Devices.vue
│  │  └─ Monitor.vue
│  ├─ components/
│  │  └─ Layout.vue
│  └─ api/
│     ├─ types.ts
│     ├─ dashboard.ts
│     └─ devices.ts
├─ package.json
├─ tsconfig.json
├─ vite.config.ts
└─ index.html
```

## 八、验收标准

- 项目可启动 `npm run dev`
- 路由可正常切换
- 仪表盘展示统计数据
- 设备列表可加载和刷新
- 页面样式基本完整
