/**
 * 前端集成测试
 *
 * 测试 Store 层面的集成流程，验证多个 action 协同工作时的状态一致性。
 * 测试环境为 node（非 jsdom），聚焦于数据/逻辑层。
 *
 * 需求: 8.2, 9.1, 9.2
 */

import { describe, it, expect, beforeEach } from "vitest";
import { useAppStore } from "@/store";
import type { PageId, Category, Project, Client, BuildRecord, ModuleInfo } from "@/types";

// ========== 测试辅助 ==========

/** 重置 store 到初始状态 */
function resetStore() {
  useAppStore.setState({
    currentPage: "projects",
    categories: [],
    projects: [],
    selectedProjectId: null,
    clients: [],
    buildRecords: [],
    projectPath: null,
    coreFiles: [],
    modules: [],
    selectedModules: new Set<string>(),
    clientName: "",
    isBuilding: false,
    buildResult: null,
  });
}

/** 所有有效页面 ID */
const ALL_PAGES: PageId[] = ["projects", "build", "settings", "about"];

/** 测试用分类数据 */
const mockCategories: Category[] = [
  { id: 1, name: "前端", description: "前端项目", created_at: "2024-01-01" },
  { id: 2, name: "后端", description: "后端服务", created_at: "2024-01-02" },
];

/** 测试用项目数据 */
const mockProjects: Project[] = [
  { id: 1, name: "Vue3 管理后台", category_id: 1, repo_path: "/projects/admin", tech_stack_type: "vue3", created_at: "2024-01-01", updated_at: "2024-01-01" },
  { id: 2, name: "FastAPI 服务", category_id: 2, repo_path: "/projects/api", tech_stack_type: "fastapi", created_at: "2024-01-02", updated_at: "2024-01-02" },
];

/** 测试用客户数据 */
const mockClients: Client[] = [
  { id: 1, name: "客户甲", created_at: "2024-01-01" },
  { id: 2, name: "客户乙", created_at: "2024-01-02" },
];

/** 测试用模块数据 */
const mockModules: ModuleInfo[] = [
  { name: "auth", path: "/projects/api/modules/auth" },
  { name: "billing", path: "/projects/api/modules/billing" },
  { name: "reports", path: "/projects/api/modules/reports" },
];

/** 测试用构建记录数据 */
const mockBuildRecords: BuildRecord[] = [
  { id: 1, project_id: 2, client_id: 1, selected_modules: '["auth","billing"]', output_path: "/output/客户甲_api.zip", created_at: "2024-03-01" },
];

// ========== 集成测试 ==========

describe("页面导航流程集成测试", () => {
  beforeEach(() => {
    resetStore();
  });

  it("默认页面应为 projects（需求 8.2）", () => {
    const state = useAppStore.getState();
    expect(state.currentPage).toBe("projects");
  });

  it("应支持在所有页面之间顺序导航", () => {
    // 模拟用户依次点击每个导航项
    for (const page of ALL_PAGES) {
      useAppStore.getState().setCurrentPage(page);
      expect(useAppStore.getState().currentPage).toBe(page);
    }
  });

  it("应支持从任意页面导航到任意其他页面", () => {
    // 验证所有页面组合的导航
    for (const from of ALL_PAGES) {
      for (const to of ALL_PAGES) {
        useAppStore.getState().setCurrentPage(from);
        expect(useAppStore.getState().currentPage).toBe(from);

        useAppStore.getState().setCurrentPage(to);
        expect(useAppStore.getState().currentPage).toBe(to);
      }
    }
  });

  it("页面切换不应影响项目管理状态", () => {
    // 先设置项目管理数据
    const store = useAppStore.getState();
    store.setCategories(mockCategories);
    store.setProjects(mockProjects);
    store.setSelectedProjectId(1);

    // 在多个页面之间切换
    for (const page of ALL_PAGES) {
      useAppStore.getState().setCurrentPage(page);
    }

    // 验证项目管理数据未被影响
    const state = useAppStore.getState();
    expect(state.categories).toEqual(mockCategories);
    expect(state.projects).toEqual(mockProjects);
    expect(state.selectedProjectId).toBe(1);
  });

  it("页面切换不应影响模块选择和构建状态", () => {
    // 先设置模块和构建状态
    const store = useAppStore.getState();
    store.setModules(mockModules);
    store.toggleModule("auth");
    store.toggleModule("billing");
    store.setClientName("客户甲");

    // 在多个页面之间切换
    useAppStore.getState().setCurrentPage("settings");
    useAppStore.getState().setCurrentPage("about");
    useAppStore.getState().setCurrentPage("build");

    // 验证模块和构建状态未被影响
    const state = useAppStore.getState();
    expect(state.modules).toEqual(mockModules);
    expect(state.selectedModules.has("auth")).toBe(true);
    expect(state.selectedModules.has("billing")).toBe(true);
    expect(state.selectedModules.has("reports")).toBe(false);
    expect(state.clientName).toBe("客户甲");
  });

  it("连续快速切换页面后状态应保持一致", () => {
    // 模拟快速连续切换
    useAppStore.getState().setCurrentPage("build");
    useAppStore.getState().setCurrentPage("settings");
    useAppStore.getState().setCurrentPage("about");
    useAppStore.getState().setCurrentPage("projects");
    useAppStore.getState().setCurrentPage("build");

    expect(useAppStore.getState().currentPage).toBe("build");
  });
});

describe("项目创建到构建完整流程集成测试", () => {
  beforeEach(() => {
    resetStore();
  });

  it("完整流程：项目管理 → 选择项目 → 切换到构建页 → 加载模块 → 选择模块 → 构建（需求 9.1, 9.2）", () => {
    const store = useAppStore.getState();

    // 步骤 1：在项目管理页面加载分类和项目数据
    expect(useAppStore.getState().currentPage).toBe("projects");
    store.setCategories(mockCategories);
    store.setProjects(mockProjects);

    // 验证数据已加载
    expect(useAppStore.getState().categories).toHaveLength(2);
    expect(useAppStore.getState().projects).toHaveLength(2);

    // 步骤 2：用户选择一个项目（模拟点击项目卡片）
    useAppStore.getState().setSelectedProjectId(2); // 选择 FastAPI 服务
    expect(useAppStore.getState().selectedProjectId).toBe(2);

    // 步骤 3：导航到构建页面
    useAppStore.getState().setCurrentPage("build");
    expect(useAppStore.getState().currentPage).toBe("build");

    // 验证选中的项目 ID 在页面切换后仍然保持
    expect(useAppStore.getState().selectedProjectId).toBe(2);

    // 步骤 4：加载关联客户
    useAppStore.getState().setClients(mockClients);
    expect(useAppStore.getState().clients).toHaveLength(2);

    // 步骤 5：扫描并加载模块（模拟 scan_project_modules 返回结果）
    useAppStore.getState().setModules(mockModules);
    expect(useAppStore.getState().modules).toHaveLength(3);

    // 步骤 6：选择要交付的模块
    useAppStore.getState().toggleModule("auth");
    useAppStore.getState().toggleModule("billing");
    expect(useAppStore.getState().selectedModules.size).toBe(2);
    expect(useAppStore.getState().selectedModules.has("auth")).toBe(true);
    expect(useAppStore.getState().selectedModules.has("billing")).toBe(true);

    // 步骤 7：设置客户名称并开始构建
    useAppStore.getState().setClientName("客户甲");
    useAppStore.getState().setBuildingState(true);
    expect(useAppStore.getState().isBuilding).toBe(true);

    // 步骤 8：构建完成，设置结果
    useAppStore.getState().setBuildingState(false);
    useAppStore.getState().setBuildResult({
      zip_path: "/output/客户甲_api.zip",
      client_name: "客户甲",
      module_count: 2,
    });

    // 步骤 9：添加构建记录到历史
    useAppStore.getState().setBuildRecords(mockBuildRecords);

    // 最终验证：所有状态应保持一致
    const finalState = useAppStore.getState();
    expect(finalState.currentPage).toBe("build");
    expect(finalState.selectedProjectId).toBe(2);
    expect(finalState.clients).toHaveLength(2);
    expect(finalState.modules).toHaveLength(3);
    expect(finalState.selectedModules.size).toBe(2);
    expect(finalState.isBuilding).toBe(false);
    expect(finalState.buildResult).not.toBeNull();
    expect(finalState.buildResult?.module_count).toBe(2);
    expect(finalState.buildRecords).toHaveLength(1);
  });

  it("选择项目后切换到构建页面，selectedProjectId 应保持不变", () => {
    // 在项目管理页面选择项目
    useAppStore.getState().setProjects(mockProjects);
    useAppStore.getState().setSelectedProjectId(1);

    // 切换到构建页面
    useAppStore.getState().setCurrentPage("build");

    // 验证 selectedProjectId 未丢失
    expect(useAppStore.getState().selectedProjectId).toBe(1);
    expect(useAppStore.getState().currentPage).toBe("build");
  });

  it("切换选中项目后应能重新加载模块和客户", () => {
    // 初始选择项目 1
    useAppStore.getState().setSelectedProjectId(1);
    useAppStore.getState().setModules([{ name: "dashboard", path: "/views/dashboard" }]);
    useAppStore.getState().setClients([{ id: 1, name: "客户甲", created_at: "2024-01-01" }]);
    useAppStore.getState().toggleModule("dashboard");

    // 验证项目 1 的状态
    expect(useAppStore.getState().selectedProjectId).toBe(1);
    expect(useAppStore.getState().modules).toHaveLength(1);
    expect(useAppStore.getState().selectedModules.has("dashboard")).toBe(true);

    // 切换到项目 2（模拟用户在构建页面切换项目选择器）
    useAppStore.getState().setSelectedProjectId(2);
    // 重新加载项目 2 的模块和客户
    useAppStore.getState().setModules(mockModules);
    useAppStore.getState().setClients(mockClients);
    // 清除之前的模块选择
    useAppStore.setState({ selectedModules: new Set<string>() });

    // 验证项目 2 的状态
    expect(useAppStore.getState().selectedProjectId).toBe(2);
    expect(useAppStore.getState().modules).toHaveLength(3);
    expect(useAppStore.getState().selectedModules.size).toBe(0);
    expect(useAppStore.getState().clients).toHaveLength(2);
  });

  it("构建完成后返回项目管理页面，再回到构建页面时状态应保持", () => {
    // 在构建页面完成一次构建
    useAppStore.getState().setCurrentPage("build");
    useAppStore.getState().setSelectedProjectId(2);
    useAppStore.getState().setModules(mockModules);
    useAppStore.getState().toggleModule("auth");
    useAppStore.getState().setBuildResult({
      zip_path: "/output/test.zip",
      client_name: "客户甲",
      module_count: 1,
    });
    useAppStore.getState().setBuildRecords(mockBuildRecords);

    // 切换到项目管理页面
    useAppStore.getState().setCurrentPage("projects");
    expect(useAppStore.getState().currentPage).toBe("projects");

    // 再切换回构建页面
    useAppStore.getState().setCurrentPage("build");

    // 验证构建相关状态仍然保持
    const state = useAppStore.getState();
    expect(state.currentPage).toBe("build");
    expect(state.selectedProjectId).toBe(2);
    expect(state.modules).toHaveLength(3);
    expect(state.selectedModules.has("auth")).toBe(true);
    expect(state.buildResult).not.toBeNull();
    expect(state.buildRecords).toHaveLength(1);
  });

  it("reset 应清除项目管理和构建状态，但保留当前页面", () => {
    // 设置完整的工作状态
    useAppStore.getState().setCurrentPage("build");
    useAppStore.getState().setCategories(mockCategories);
    useAppStore.getState().setProjects(mockProjects);
    useAppStore.getState().setSelectedProjectId(2);
    useAppStore.getState().setClients(mockClients);
    useAppStore.getState().setModules(mockModules);
    useAppStore.getState().toggleModule("auth");
    useAppStore.getState().setBuildRecords(mockBuildRecords);

    // 执行 reset
    useAppStore.getState().reset();

    // 验证当前页面保留
    expect(useAppStore.getState().currentPage).toBe("build");

    // 验证项目管理和构建状态已清除
    const state = useAppStore.getState();
    expect(state.categories).toEqual([]);
    expect(state.projects).toEqual([]);
    expect(state.selectedProjectId).toBeNull();
    expect(state.clients).toEqual([]);
    expect(state.buildRecords).toEqual([]);
    expect(state.modules).toEqual([]);
    expect(state.selectedModules.size).toBe(0);
  });
});
