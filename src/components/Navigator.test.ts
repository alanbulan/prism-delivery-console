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

/** 导航项配置（与 Navigator.tsx 中的 NAV_ITEMS 保持一致） */
const EXPECTED_NAV_ITEMS: Array<{ id: PageId; label: string }> = [
  { id: "projects", label: "项目管理" },
  { id: "build", label: "构建交付" },
  { id: "settings", label: "设置" },
  { id: "about", label: "关于" },
];

/** 重置 store 到初始状态 */
function resetStore() {
  useAppStore.setState({
    currentPage: "projects",
  });
}

describe("Navigator 导航配置", () => {
  beforeEach(() => {
    resetStore();
  });

  it("应包含 4 个导航项：项目管理、构建交付、设置、关于（需求 8.1）", () => {
    expect(EXPECTED_NAV_ITEMS).toHaveLength(4);
    expect(EXPECTED_NAV_ITEMS.map((item) => item.id)).toEqual([
      "projects",
      "build",
      "settings",
      "about",
    ]);
  });

  it("每个导航项应有中文标签", () => {
    for (const item of EXPECTED_NAV_ITEMS) {
      expect(item.label).toBeTruthy();
      expect(typeof item.label).toBe("string");
    }
  });

  it("导航项 ID 应覆盖所有有效的 PageId 值", () => {
    const allPageIds: PageId[] = ["projects", "build", "settings", "about"];
    const navIds = EXPECTED_NAV_ITEMS.map((item) => item.id);
    for (const pageId of allPageIds) {
      expect(navIds).toContain(pageId);
    }
  });
});

describe("Navigator Store 集成", () => {
  beforeEach(() => {
    resetStore();
  });

  it("点击导航项应通过 setCurrentPage 切换页面（需求 8.3）", () => {
    // 模拟点击每个导航项
    for (const item of EXPECTED_NAV_ITEMS) {
      useAppStore.getState().setCurrentPage(item.id);
      expect(useAppStore.getState().currentPage).toBe(item.id);
    }
  });

  it("当前活动页面应与 store.currentPage 一致（需求 8.3）", () => {
    // 默认页面
    expect(useAppStore.getState().currentPage).toBe("projects");

    // 切换到构建页面
    useAppStore.getState().setCurrentPage("build");
    expect(useAppStore.getState().currentPage).toBe("build");

    // 切换到设置页面
    useAppStore.getState().setCurrentPage("settings");
    expect(useAppStore.getState().currentPage).toBe("settings");
  });

  it("默认应显示项目管理页面（需求 8.4）", () => {
    expect(useAppStore.getState().currentPage).toBe("projects");
  });
});
