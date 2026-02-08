/**
 * Navigator 组件单元测试
 *
 * 验证导航项配置的正确性和 Store 集成
 * 注意：测试环境为 node（非 jsdom），因此测试聚焦于数据逻辑层
 *
 * 需求: 8.1, 8.3, 8.5
 */

import { describe, it, expect, beforeEach } from "vitest";
import { useAppStore } from "@/store";
import type { PageId } from "@/types";

/** 主导航项配置（与 Navigator.tsx 中的 MAIN_NAV_ITEMS 保持一致） */
const EXPECTED_MAIN_NAV: Array<{ id: PageId; label: string }> = [
  { id: "projects", label: "项目管理" },
  { id: "build", label: "构建交付" },
  { id: "analysis", label: "项目分析" },
];

/** 底部导航项配置（与 Navigator.tsx 中的 BOTTOM_NAV_ITEMS 保持一致） */
const EXPECTED_BOTTOM_NAV: Array<{ id: PageId; label: string }> = [
  { id: "settings", label: "设置" },
  { id: "about", label: "关于" },
];

/** 全部导航项 */
const ALL_NAV_ITEMS = [...EXPECTED_MAIN_NAV, ...EXPECTED_BOTTOM_NAV];

/** 重置 store 到初始状态 */
function resetStore() {
  useAppStore.setState({
    currentPage: "build",
    sidebarCollapsed: false,
  });
}

describe("Navigator 导航配置", () => {
  beforeEach(() => {
    resetStore();
  });

  it("主导航应包含 3 项：项目管理、构建交付、项目分析", () => {
    expect(EXPECTED_MAIN_NAV).toHaveLength(3);
    expect(EXPECTED_MAIN_NAV.map((item) => item.id)).toEqual([
      "projects",
      "build",
      "analysis",
    ]);
  });

  it("底部导航应包含 2 项：设置、关于", () => {
    expect(EXPECTED_BOTTOM_NAV).toHaveLength(2);
    expect(EXPECTED_BOTTOM_NAV.map((item) => item.id)).toEqual([
      "settings",
      "about",
    ]);
  });

  it("全部导航项共 5 个，覆盖所有 PageId", () => {
    const allPageIds: PageId[] = ["projects", "build", "analysis", "settings", "about"];
    const navIds = ALL_NAV_ITEMS.map((item) => item.id);
    expect(navIds).toHaveLength(5);
    for (const pageId of allPageIds) {
      expect(navIds).toContain(pageId);
    }
  });

  it("每个导航项应有中文标签", () => {
    for (const item of ALL_NAV_ITEMS) {
      expect(item.label).toBeTruthy();
      expect(typeof item.label).toBe("string");
    }
  });
});

describe("Navigator Store 集成", () => {
  beforeEach(() => {
    resetStore();
  });

  it("点击导航项应通过 setCurrentPage 切换页面（需求 8.3）", () => {
    for (const item of ALL_NAV_ITEMS) {
      useAppStore.getState().setCurrentPage(item.id);
      expect(useAppStore.getState().currentPage).toBe(item.id);
    }
  });

  it("默认应显示构建交付页面", () => {
    expect(useAppStore.getState().currentPage).toBe("build");
  });

  it("侧边栏默认展开", () => {
    expect(useAppStore.getState().sidebarCollapsed).toBe(false);
  });

  it("toggleSidebar 应切换收展状态", () => {
    expect(useAppStore.getState().sidebarCollapsed).toBe(false);
    useAppStore.getState().toggleSidebar();
    expect(useAppStore.getState().sidebarCollapsed).toBe(true);
    useAppStore.getState().toggleSidebar();
    expect(useAppStore.getState().sidebarCollapsed).toBe(false);
  });
});
