/**
 * 关于页面 (AboutPage)
 *
 * 职责：
 * - 展示应用名称、版本号、技术栈信息和团队信息
 * - 版本号通过 Tauri getVersion API 动态读取
 * - Liquid Glass 风格的精致卡片布局
 *
 * 需求: 11.1, 11.2, 11.3, 11.4
 */

import { useEffect, useState } from "react";
import { getVersion } from "@tauri-apps/api/app";
import { Layers, Users, Code2, Heart } from "lucide-react";

/** 技术栈条目 */
const TECH_STACK = [
  { name: "Tauri", version: "v2", desc: "跨平台桌面框架" },
  { name: "React", version: "19", desc: "UI 渲染引擎" },
  { name: "TypeScript", version: "5.8", desc: "类型安全" },
  { name: "Rust", version: "stable", desc: "后端核心逻辑" },
  { name: "Zustand", version: "5", desc: "状态管理" },
  { name: "Tailwind CSS", version: "4", desc: "原子化样式" },
] as const;

export function AboutPage() {
  /** 应用版本号，从 Tauri 配置中读取 */
  const [version, setVersion] = useState<string>("");

  useEffect(() => {
    getVersion().then(setVersion).catch(() => setVersion("unknown"));
  }, []);

  return (
    <div className="flex flex-1 flex-col overflow-hidden">
      {/* 页面标题栏 */}
      <header className="glass-subtle flex items-center px-5 py-3">
        <h2 className="text-base font-semibold text-foreground">关于</h2>
      </header>

      {/* 主内容区 */}
      <main className="flex flex-1 items-start justify-center overflow-auto p-6">
        <div className="flex w-full max-w-2xl flex-col gap-5">

          {/* 应用标识卡片 */}
          <section className="glass-panel flex flex-col items-center gap-4 p-8">
            {/* 应用图标占位 - 使用 Prism 棱镜概念 */}
            <div className="flex h-16 w-16 items-center justify-center rounded-2xl bg-gradient-to-br from-blue-500/20 via-indigo-500/20 to-purple-500/20 shadow-inner">
              <div className="h-8 w-8 rotate-45 rounded-lg bg-gradient-to-br from-blue-400 to-indigo-500 opacity-80" />
            </div>

            <div className="flex flex-col items-center gap-1">
              <h1 className="text-xl font-bold text-foreground">
                Prism Delivery Console
              </h1>
              <p className="text-sm text-muted-foreground">
                多项目交付包构建管理工具
              </p>
            </div>

            {/* 版本标签 */}
            {version && (
              <span className="rounded-full bg-accent px-4 py-1 text-xs font-medium text-accent-foreground">
                v{version}
              </span>
            )}
          </section>

          {/* 技术栈卡片 */}
          <section className="glass flex flex-col gap-3 p-5">
            <div className="flex items-center gap-2">
              <Layers className="h-4 w-4 text-blue-500/70" />
              <h3 className="text-sm font-semibold text-foreground">技术栈</h3>
            </div>
            <div className="grid grid-cols-2 gap-2 sm:grid-cols-3">
              {TECH_STACK.map((tech) => (
                <div
                  key={tech.name}
                  className="glass-subtle flex flex-col gap-0.5 px-3 py-2.5"
                >
                  <div className="flex items-baseline gap-1.5">
                    <span className="text-sm font-medium text-foreground">
                      {tech.name}
                    </span>
                    <span className="text-xs text-muted-foreground">
                      {tech.version}
                    </span>
                  </div>
                  <span className="text-xs text-muted-foreground/70">
                    {tech.desc}
                  </span>
                </div>
              ))}
            </div>
          </section>

          {/* 项目信息卡片 */}
          <div className="grid grid-cols-2 gap-4">
            {/* 开发团队 */}
            <section className="glass flex flex-col gap-3 p-5">
              <div className="flex items-center gap-2">
                <Users className="h-4 w-4 text-indigo-500/70" />
                <h3 className="text-sm font-semibold text-foreground">
                  开发团队
                </h3>
              </div>
              <p className="text-sm text-muted-foreground">Prism Team</p>
            </section>

            {/* 开源协议 */}
            <section className="glass flex flex-col gap-3 p-5">
              <div className="flex items-center gap-2">
                <Code2 className="h-4 w-4 text-emerald-500/70" />
                <h3 className="text-sm font-semibold text-foreground">
                  项目信息
                </h3>
              </div>
              <p className="text-sm text-muted-foreground">内部工具 · 私有项目</p>
            </section>
          </div>

          {/* 底部致谢 */}
          <div className="flex items-center justify-center gap-1.5 py-2 text-xs text-muted-foreground/60">
            <span>Built with</span>
            <Heart className="h-3 w-3 text-red-400/60" />
            <span>using Tauri + React</span>
          </div>

        </div>
      </main>
    </div>
  );
}
