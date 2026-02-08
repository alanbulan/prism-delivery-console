/**
 * 顶层布局组件 (AppShell)
 *
 * 整体布局结构：
 * ┌─────────────────────────────────────────┐
 * │ TitleBar (全宽, h-9)                     │
 * ├──────────┬──────────────────────────────┤
 * │ Navigator│ 页面内容区                    │
 * │ (w-48)   │ (flex-1, 可滚动)             │
 * │          │                              │
 * └──────────┴──────────────────────────────┘
 *
 * 职责：
 * - 组合 TitleBar + Navigator 侧边栏 + 页面内容区
 * - 根据 store.currentPage 渲染对应页面组件
 * - 包含背景装饰层和 Toast 通知
 *
 * 需求: 8.2, 8.4
 */

import { Toaster } from "sonner";
import { useAppStore } from "@/store";
import { TitleBar } from "@/components/TitleBar";
import { Navigator } from "@/components/Navigator";
import { ProjectPage } from "@/pages/projects";
import { BuildPage } from "@/pages/build";
import { AnalysisPage } from "@/pages/analysis";
import { SettingsPage } from "@/pages/SettingsPage";
import { AboutPage } from "@/pages/AboutPage";
import type { PageId } from "@/types";

/** 页面组件映射：根据 PageId 返回对应的页面内容 */
function PageContent({ pageId }: { pageId: PageId }) {
  switch (pageId) {
    case "projects":
      return <ProjectPage />;
    case "build":
      return <BuildPage />;
    case "analysis":
      return <AnalysisPage />;
    case "settings":
      return <SettingsPage />;
    case "about":
      return <AboutPage />;
  }
}

export function AppShell() {
  /** 从 Store 获取当前页面标识 */
  const currentPage = useAppStore((s) => s.currentPage);

  return (
    <div className="relative flex h-screen flex-col bg-background">
      {/* 背景装饰层：渐变 + 光斑效果，增强玻璃质感 */}
      <div className="pointer-events-none fixed inset-0 -z-10 overflow-hidden">
        {/* 主渐变背景 */}
        <div className="absolute inset-0 bg-gradient-to-br from-blue-50/80 via-slate-50/60 to-indigo-50/70" />
        {/* 顶部光斑装饰 */}
        <div className="absolute -top-32 -right-32 h-96 w-96 rounded-full bg-blue-100/40 blur-3xl" />
        {/* 底部光斑装饰 */}
        <div className="absolute -bottom-32 -left-32 h-96 w-96 rounded-full bg-indigo-100/30 blur-3xl" />
        {/* 中间微弱光斑 */}
        <div className="absolute top-1/2 left-1/2 h-64 w-64 -translate-x-1/2 -translate-y-1/2 rounded-full bg-sky-100/20 blur-3xl" />
      </div>

      {/* 自定义标题栏（全宽, h-9） */}
      <TitleBar />

      {/* 主体区域：侧边栏导航 + 页面内容 */}
      <div className="flex flex-1 overflow-hidden">
        {/* 侧边栏导航（宽度随收展状态变化） */}
        <Navigator />

        {/* 页面内容区（可滚动） */}
        <div className="flex flex-1 flex-col overflow-auto">
          <PageContent pageId={currentPage} />
        </div>
      </div>

      {/* Toast 通知容器 */}
      <Toaster
        position="top-right"
        richColors
        toastOptions={{
          className: "glass",
        }}
      />
    </div>
  );
}
