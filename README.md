<p align="center">
  <img src="src-tauri/icons/prism-icon.svg" width="120" alt="Prism Logo" />
</p>

<h1 align="center">Prism Delivery Console</h1>

<p align="center">
  <strong>多项目交付包构建 & 智能分析桌面工具</strong>
</p>

<p align="center">
  <a href="https://github.com/alanbulan/prism-delivery-console/releases/latest">
    <img src="https://img.shields.io/github/v/release/alanbulan/prism-delivery-console?style=flat-square&color=blue" alt="Latest Release" />
  </a>
  <a href="https://github.com/alanbulan/prism-delivery-console/actions/workflows/release.yml">
    <img src="https://img.shields.io/github/actions/workflow/status/alanbulan/prism-delivery-console/release.yml?style=flat-square&label=build" alt="Build Status" />
  </a>
  <a href="LICENSE">
    <img src="https://img.shields.io/github/license/alanbulan/prism-delivery-console?style=flat-square" alt="License" />
  </a>
  <img src="https://img.shields.io/badge/platform-Windows-0078D6?style=flat-square&logo=windows" alt="Platform" />
</p>

<p align="center">
  <a href="#功能特性">功能特性</a> •
  <a href="#快速开始">快速开始</a> •
  <a href="#技术架构">技术架构</a> •
  <a href="#开发指南">开发指南</a> •
  <a href="#发布流程">发布流程</a>
</p>

---

## 简介

Prism Delivery Console 是一款基于 Tauri v2 的跨平台桌面应用，专为多项目、多技术栈的交付包构建场景设计。它将项目管理、智能构建、代码分析整合在一个轻量级原生应用中，帮助开发团队高效完成交付工作。

> 💡 "Prism"（棱镜）—— 一束白光经过棱镜折射出七彩光谱，正如一个项目经过 Prism 拆解出清晰的模块、依赖和交付物。

## 功能特性

### 🏗️ 智能构建

- **多技术栈支持** — FastAPI、Vue3 项目的模块扫描与交付包构建
- **Import 路径重写** — 自动分析并重写 Python/JS 模块的 import 路径
- **实时构建日志** — 通过 Tauri Event 实时推送构建进度到前端
- **构建历史管理** — 完整的构建记录追踪，支持清理与回溯
- **客户模块配置** — 按客户维度保存模块选择，一键复用

### 📊 项目分析

- **文件索引** — 增量哈希检测，仅处理变更文件
- **依赖拓扑** — D3.js 力导向图 + 树形视图，支持文件/目录粒度切换
- **语义搜索** — 基于 Embedding 向量的代码语义检索（SQLite BLOB + 余弦相似度）
- **AI 报告** — 静态签名提取 + LLM 生成项目分析报告
- **项目概览** — 语言分布、文件统计、代码规模一目了然

### 📁 项目管理

- **分类管理** — 自定义项目分类，支持描述与排序
- **多项目切换** — 快速在不同项目间切换工作上下文
- **技术栈识别** — 自动检测项目技术栈类型

### ⚙️ 系统能力

- **自动更新** — 基于 Tauri Updater 的应用内自动更新
- **LLM 集成** — 可配置的 OpenAI 兼容 API（Chat + Embedding 模型）
- **原生性能** — Rust 后端，启动快、内存占用低

## 截图预览

> 截图待补充

## 快速开始

### 安装

前往 [Releases](https://github.com/alanbulan/prism-delivery-console/releases/latest) 下载最新版本的安装包：

- `.msi` — Windows Installer 安装包
- `.exe` — NSIS 安装包

安装后应用会自动检查更新，无需手动下载后续版本。

### 从源码构建

#### 前置要求

- [Node.js](https://nodejs.org/) >= 18 (LTS)
- [Rust](https://rustup.rs/) >= 1.77 (stable)
- [Tauri CLI](https://v2.tauri.app/start/prerequisites/) v2

```bash
# 克隆仓库
git clone https://github.com/alanbulan/prism-delivery-console.git
cd prism-delivery-console

# 安装前端依赖
npm ci

# 开发模式（热重载）
npm run tauri dev

# 构建生产版本
npm run tauri build
```

## 技术架构

```
┌─────────────────────────────────────────────────┐
│                   Frontend                       │
│  React 19 + TypeScript + Tailwind CSS + Zustand  │
│  D3.js (拓扑图) · Sonner (通知) · Lucide (图标)   │
├─────────────────────────────────────────────────┤
│               Tauri v2 IPC Bridge                │
├─────────────────────────────────────────────────┤
│                    Backend                       │
│              Rust (Clean Architecture)           │
│                                                  │
│  commands/     接口层 — 参数校验 + 调用 services   │
│  services/     业务层 — 扫描/构建/分析/LLM        │
│  models/       数据层 — DTO 定义                  │
│  utils/        工具层 — 统一错误处理              │
│                                                  │
│  SQLite (rusqlite) · reqwest · sha2 · regex      │
└─────────────────────────────────────────────────┘
```

### 后端分层

| 层级 | 目录 | 职责 |
|------|------|------|
| 接口层 | `src-tauri/src/commands/` | 接收前端参数，调用 services，返回 Result |
| 业务层 | `src-tauri/src/services/` | 纯 Rust 核心逻辑，不依赖 Tauri API |
| 数据层 | `src-tauri/src/models/` | 数据结构定义 (Serialize/Deserialize) |
| 工具层 | `src-tauri/src/utils/` | 统一错误类型 (AppError + thiserror) |

### 前端结构

```
src/
├── components/        全局共享组件
├── pages/
│   ├── build/         构建页（选择器 + 历史 + 日志）
│   ├── projects/      项目管理页（分类 + CRUD）
│   ├── analysis/      分析页（概览 + 文件 + 拓扑 + 搜索）
│   ├── SettingsPage   设置页（LLM + Embedding + 自动索引）
│   └── AboutPage      关于页（版本 + 更新 + 技术栈）
├── store.ts           Zustand 全局状态
└── types.ts           TypeScript 类型定义
```

## 开发指南

### 项目命令

```bash
# 前端开发服务器
npm run dev

# TypeScript 类型检查 + Vite 构建
npm run build

# 运行前端测试
npx vitest --run

# Rust 后端测试
cargo test --manifest-path src-tauri/Cargo.toml

# Tauri 开发模式（前后端联调）
npm run tauri dev

# Tauri 生产构建
npm run tauri build
```

### 添加新功能

1. 在 `src-tauri/src/models/` 定义数据结构
2. 在 `src-tauri/src/services/` 实现业务逻辑
3. 在 `src-tauri/src/commands/` 创建 Tauri command（薄接口层）
4. 在 `src-tauri/src/lib.rs` 注册 command
5. 在前端 `src/types.ts` 定义对应 TypeScript 类型
6. 在页面 `composables/` 中封装调用逻辑
7. 在页面 `components/` 中实现 UI

## 发布流程

项目使用 GitHub Actions 自动构建发布：

1. 更新版本号（`package.json` + `Cargo.toml` + `tauri.conf.json`）
2. 提交并推送到 `main` 分支
3. 创建并推送 tag：`git tag v0.x.0 && git push origin v0.x.0`
4. GitHub Actions 自动触发构建，生成安装包并发布到 Releases
5. 已安装的客户端会通过 Updater 自动检测到新版本

## 许可证

[MIT](LICENSE)

---

<p align="center">
  Built with Tauri + React + Rust
</p>
