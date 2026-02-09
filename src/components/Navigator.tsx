/**
 * 侧边栏导航组件
 *
 * 提供 5 个页面导航入口，分为主导航和底部导航：
 * 主导航：项目管理、构建交付、项目分析
 * 底部：设置、关于
 *
 * 功能特性：
 * - 支持收展（collapsed/expanded）
 * - 收起时只显示图标，hover 显示 tooltip
 * - 使用 Lucide 图标标识每个导航项
 * - 高亮当前活动页面
 * - 应用 Liquid Glass glass-panel 样式
 */

import { FolderKanban, Package, Settings, Info, PanelLeftClose, PanelLeftOpen, Network, Blocks } from "lucide-react";
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

/** 主导航项（顶部） */
const MAIN_NAV_ITEMS: NavItem[] = [
  { id: "projects", label: "项目管理", icon: FolderKanban },
  { id: "build", label: "构建交付", icon: Package },
  { id: "analysis", label: "项目分析", icon: Network },
  { id: "templates", label: "模板管理", icon: Blocks },
];

/** 底部导航项 */
const BOTTOM_NAV_ITEMS: NavItem[] = [
  { id: "settings", label: "设置", icon: Settings },
  { id: "about", label: "关于", icon: Info },
];

/** 渲染单个导航按钮 */
function NavButton({
  item,
  isActive,
  collapsed,
  onClick,
}: {
  item: NavItem;
  isActive: boolean;
  collapsed: boolean;
  onClick: () => void;
}) {
  const Icon = item.icon;

  return (
    <button
      key={item.id}
      type="button"
      onClick={onClick}
      title={collapsed ? item.label : undefined}
      className={`flex items-center gap-3 rounded-lg px-3 py-2.5 text-sm transition-colors ${
        collapsed ? "justify-center" : ""
      } ${
        isActive
          ? "bg-primary text-primary-foreground font-medium shadow-sm"
          : "text-muted-foreground hover:bg-accent hover:text-accent-foreground"
      }`}
      aria-current={isActive ? "page" : undefined}
    >
      <Icon className="h-4 w-4 shrink-0" />
      {!collapsed && <span>{item.label}</span>}
    </button>
  );
}

export function Navigator() {
  /** 从 Store 获取当前页面、侧边栏状态和操作方法 */
  const currentPage = useAppStore((s) => s.currentPage);
  const setCurrentPage = useAppStore((s) => s.setCurrentPage);
  const collapsed = useAppStore((s) => s.sidebarCollapsed);
  const toggleSidebar = useAppStore((s) => s.toggleSidebar);

  const CollapseIcon = collapsed ? PanelLeftOpen : PanelLeftClose;

  return (
    <nav
      className={`glass-panel flex shrink-0 flex-col p-3 transition-all duration-200 ${
        collapsed ? "w-14" : "w-48"
      }`}
    >
      {/* 主导航区域 */}
      <div className="flex flex-col gap-1">
        {MAIN_NAV_ITEMS.map((item) => (
          <NavButton
            key={item.id}
            item={item}
            isActive={currentPage === item.id}
            collapsed={collapsed}
            onClick={() => setCurrentPage(item.id)}
          />
        ))}
      </div>

      {/* 弹性间距，将底部导航推到最下方 */}
      <div className="flex-1" />

      {/* 底部导航区域 */}
      <div className="flex flex-col gap-1">
        {BOTTOM_NAV_ITEMS.map((item) => (
          <NavButton
            key={item.id}
            item={item}
            isActive={currentPage === item.id}
            collapsed={collapsed}
            onClick={() => setCurrentPage(item.id)}
          />
        ))}

        {/* 收展按钮 */}
        <button
          type="button"
          onClick={toggleSidebar}
          title={collapsed ? "展开侧边栏" : "收起侧边栏"}
          className={`flex items-center gap-3 rounded-lg px-3 py-2.5 text-sm text-muted-foreground transition-colors hover:bg-accent hover:text-accent-foreground ${
            collapsed ? "justify-center" : ""
          }`}
        >
          <CollapseIcon className="h-4 w-4 shrink-0" />
          {!collapsed && <span>收起</span>}
        </button>
      </div>
    </nav>
  );
}
