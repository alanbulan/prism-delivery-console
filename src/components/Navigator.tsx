/**
 * 侧边栏导航组件
 *
 * 提供 4 个页面导航入口：
 * - 项目管理 (projects)
 * - 构建交付 (build)
 * - 设置 (settings)
 * - 关于 (about)
 *
 * 功能特性：
 * - 使用 Lucide 图标标识每个导航项
 * - 高亮当前活动页面
 * - 应用 Liquid Glass glass-panel 样式
 *
 * 需求: 8.1, 8.3, 8.5
 */

import { FolderKanban, Package, Settings, Info, Zap } from "lucide-react";
import { useAppStore } from "@/store";
import type { PageId } from "@/types";
import type { LucideIcon } from "lucide-react";

/** 导航项配置 */
interface NavItem {
  /** 页面标识 */
  id: PageId;
  /** 显示名称 */
  label: string;
  /** Lucide 图标组件 */
  icon: LucideIcon;
}

/** 导航项列表定义 */
const NAV_ITEMS: NavItem[] = [
  { id: "projects", label: "项目管理", icon: FolderKanban },
  { id: "build", label: "构建交付", icon: Package },
  { id: "quick-build", label: "快速构建", icon: Zap },
  { id: "settings", label: "设置", icon: Settings },
  { id: "about", label: "关于", icon: Info },
];

export function Navigator() {
  /** 从 Store 获取当前页面和页面切换方法 */
  const currentPage = useAppStore((s) => s.currentPage);
  const setCurrentPage = useAppStore((s) => s.setCurrentPage);

  return (
    <nav className="glass-panel flex w-48 shrink-0 flex-col gap-1 p-3">
      {NAV_ITEMS.map((item) => {
        const isActive = currentPage === item.id;
        const Icon = item.icon;

        return (
          <button
            key={item.id}
            type="button"
            onClick={() => setCurrentPage(item.id)}
            className={`flex items-center gap-3 rounded-lg px-3 py-2.5 text-sm transition-colors ${
              isActive
                ? "bg-primary text-primary-foreground font-medium shadow-sm"
                : "text-muted-foreground hover:bg-accent hover:text-accent-foreground"
            }`}
            aria-current={isActive ? "page" : undefined}
          >
            <Icon className="h-4 w-4 shrink-0" />
            <span>{item.label}</span>
          </button>
        );
      })}
    </nav>
  );
}
