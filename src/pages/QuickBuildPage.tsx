/**
 * 快速构建页面 (QuickBuildPage)
 *
 * V1 工作流：直接选择文件夹 → 查看核心文件 → 选择模块 → 输入客户名 → 构建
 * 无需通过数据库管理项目，适合一次性快速构建场景
 *
 * 布局结构：
 * ┌──────────────────────────────────────────────────┐
 * │ 快速构建                                          │
 * ├──────────────────────────────────────────────────┤
 * │ [ProjectSelector] 打开项目 + 路径显示             │
 * ├────────────┬─────────────────────────────────────┤
 * │ CoreFiles  │ ModuleSelector                      │
 * │ (核心文件) │ (模块选择)                           │
 * ├────────────┴─────────────────────────────────────┤
 * │ [BuildPanel] 客户名输入 + 构建按钮               │
 * └──────────────────────────────────────────────────┘
 */

import { ProjectSelector } from "@/components/ProjectSelector";
import { CoreFilesList } from "@/components/CoreFilesList";
import { ModuleSelector } from "@/components/ModuleSelector";
import { BuildPanel } from "@/components/BuildPanel";

export function QuickBuildPage() {
  return (
    <div className="flex flex-1 flex-col overflow-hidden">
      {/* 页面标题栏 */}
      <header className="glass-subtle flex items-center px-5 py-3">
        <h2 className="text-base font-semibold text-foreground">快速构建</h2>
      </header>

      {/* 项目选择器 */}
      <div className="glass-subtle mx-4 mt-4 px-4 py-3">
        <ProjectSelector />
      </div>

      {/* 主内容区：左侧核心文件 + 右侧模块选择 */}
      <main className="flex flex-1 gap-4 overflow-hidden p-4">
        {/* 左侧：核心架构文件列表 */}
        <aside className="glass-panel flex w-56 shrink-0 flex-col p-3">
          <h3 className="mb-2 px-1 text-sm font-semibold text-foreground">
            核心文件
          </h3>
          <CoreFilesList />
        </aside>

        {/* 右侧：模块选择区域 */}
        <section className="glass-panel flex flex-1 flex-col p-4">
          <h3 className="mb-3 text-sm font-semibold text-foreground">
            业务模块
          </h3>
          <ModuleSelector />
        </section>
      </main>

      {/* 底部：构建面板 */}
      <footer className="glass-subtle mx-4 mb-4 px-4 py-3">
        <BuildPanel />
      </footer>
    </div>
  );
}
