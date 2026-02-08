/**
 * 自定义标题栏组件
 *
 * 替代系统默认标题栏，提供：
 * - 窗口拖拽区域（data-tauri-drag-region）
 * - 最小化、最大化/还原、关闭按钮
 * - 应用名称和当前页面上下文显示
 * - Liquid Glass 视觉风格
 *
 * 需求: 7.2, 7.3, 7.4, 7.5, 7.6
 */

import { useEffect, useState, useCallback } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Minus, Square, Copy, X } from "lucide-react";
import { useAppStore } from "@/store";
import type { PageId } from "@/types";

/** 页面标识到中文名称的映射 */
const PAGE_NAME_MAP: Record<PageId, string> = {
  projects: "项目管理",
  build: "构建交付",
  analysis: "项目分析",
  settings: "设置",
  about: "关于",
};

/** 获取 Tauri 窗口实例（模块级缓存，避免重复调用） */
const appWindow = getCurrentWindow();

export function TitleBar() {
  /** 窗口是否处于最大化状态 */
  const [isMaximized, setIsMaximized] = useState(false);

  /** 从 Store 获取当前页面标识 */
  const currentPage = useAppStore((s) => s.currentPage);

  /** 初始化最大化状态并监听窗口尺寸变化 */
  useEffect(() => {
    // 获取初始最大化状态
    appWindow.isMaximized().then(setIsMaximized);

    // 监听窗口尺寸变化，更新最大化状态
    let unlisten: (() => void) | undefined;
    const setupListener = async () => {
      unlisten = await appWindow.onResized(async () => {
        const maximized = await appWindow.isMaximized();
        setIsMaximized(maximized);
      });
    };
    setupListener();

    // 组件卸载时取消监听
    return () => {
      unlisten?.();
    };
  }, []);

  /** 最小化窗口 */
  const handleMinimize = useCallback(() => {
    appWindow.minimize();
  }, []);

  /** 切换最大化/还原状态 */
  const handleToggleMaximize = useCallback(() => {
    appWindow.toggleMaximize();
  }, []);

  /** 关闭窗口 */
  const handleClose = useCallback(() => {
    appWindow.close();
  }, []);

  return (
    <div
      data-tauri-drag-region
      className="glass-subtle flex h-9 shrink-0 select-none items-center justify-between px-3"
      style={{ appRegion: "drag", WebkitAppRegion: "drag" } as React.CSSProperties}
    >
      {/* 左侧：应用名称 + 当前页面上下文 */}
      <div data-tauri-drag-region className="flex items-center gap-2 text-xs">
        <span
          data-tauri-drag-region
          className="font-semibold text-foreground/80"
        >
          Prism Delivery Console
        </span>
        <span data-tauri-drag-region className="text-muted-foreground/60">
          /
        </span>
        <span data-tauri-drag-region className="text-muted-foreground">
          {PAGE_NAME_MAP[currentPage]}
        </span>
      </div>

      {/* 右侧：窗口控制按钮组（禁止拖拽，允许点击） */}
      <div className="flex items-center" style={{ appRegion: "no-drag", WebkitAppRegion: "no-drag" } as React.CSSProperties}>
        {/* 最小化按钮 */}
        <button
          type="button"
          onClick={handleMinimize}
          className="flex h-7 w-8 items-center justify-center rounded-sm text-muted-foreground transition-colors hover:bg-accent hover:text-foreground"
          aria-label="最小化"
        >
          <Minus className="h-3.5 w-3.5" />
        </button>

        {/* 最大化/还原按钮：根据窗口状态切换图标 */}
        <button
          type="button"
          onClick={handleToggleMaximize}
          className="flex h-7 w-8 items-center justify-center rounded-sm text-muted-foreground transition-colors hover:bg-accent hover:text-foreground"
          aria-label={isMaximized ? "还原" : "最大化"}
        >
          {isMaximized ? (
            <Copy className="h-3 w-3" />
          ) : (
            <Square className="h-3 w-3" />
          )}
        </button>

        {/* 关闭按钮：悬浮时显示红色背景 */}
        <button
          type="button"
          onClick={handleClose}
          className="flex h-7 w-8 items-center justify-center rounded-sm text-muted-foreground transition-colors hover:bg-destructive hover:text-white"
          aria-label="关闭"
        >
          <X className="h-3.5 w-3.5" />
        </button>
      </div>
    </div>
  );
}
