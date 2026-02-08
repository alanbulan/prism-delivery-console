import { describe, it, expect, beforeEach } from "vitest";
import fc from "fast-check";
import { useAppStore } from "./store";

/**
 * Zustand Store 单元测试
 * 验证 AppStore 的所有状态和 actions 行为
 */

/** 辅助函数：重置 store 到初始状态 */
function resetStore() {
  useAppStore.setState({
    currentPage: 'projects',
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

/** 测试用模块数据 */
const mockModules = [
  { name: "auth", path: "/project/modules/auth" },
  { name: "billing", path: "/project/modules/billing" },
  { name: "reports", path: "/project/modules/reports" },
];

describe("AppStore", () => {
  beforeEach(() => {
    resetStore();
  });

  describe("初始状态", () => {
    it("应具有正确的默认值", () => {
      const state = useAppStore.getState();
      // V2 导航状态
      expect(state.currentPage).toBe('projects');
      // V2 项目管理状态
      expect(state.categories).toEqual([]);
      expect(state.projects).toEqual([]);
      expect(state.selectedProjectId).toBeNull();
      expect(state.clients).toEqual([]);
      expect(state.buildRecords).toEqual([]);
      // V1 项目状态
      expect(state.projectPath).toBeNull();
      expect(state.coreFiles).toEqual([]);
      expect(state.modules).toEqual([]);
      expect(state.selectedModules).toEqual(new Set());
      expect(state.clientName).toBe("");
      expect(state.isBuilding).toBe(false);
      expect(state.buildResult).toBeNull();
    });
  });

  describe("setProject", () => {
    it("应设置项目路径和核心文件", () => {
      const { setProject } = useAppStore.getState();
      setProject("/path/to/project", ["main.py", "config/"]);

      const state = useAppStore.getState();
      expect(state.projectPath).toBe("/path/to/project");
      expect(state.coreFiles).toEqual(["main.py", "config/"]);
    });

    it("应在设置新项目时重置模块和构建状态（需求 7.2）", () => {
      const store = useAppStore.getState();
      // 先设置一些状态
      store.setProject("/old/path", ["main.py"]);
      store.setModules(mockModules);
      store.toggleModule("auth");
      store.setClientName("客户A");
      store.setBuildResult({
        zip_path: "/output/dist.zip",
        client_name: "客户A",
        module_count: 1,
      });

      // 重新设置项目
      useAppStore.getState().setProject("/new/path", ["main.py", "utils/"]);

      const state = useAppStore.getState();
      // 项目信息应更新
      expect(state.projectPath).toBe("/new/path");
      expect(state.coreFiles).toEqual(["main.py", "utils/"]);
      // 模块和构建状态应被重置
      expect(state.modules).toEqual([]);
      expect(state.selectedModules.size).toBe(0);
      expect(state.clientName).toBe("");
      expect(state.buildResult).toBeNull();
      expect(state.isBuilding).toBe(false);
    });
  });

  describe("setModules", () => {
    it("应设置模块列表", () => {
      useAppStore.getState().setModules(mockModules);
      expect(useAppStore.getState().modules).toEqual(mockModules);
    });

    it("应支持设置空模块列表", () => {
      useAppStore.getState().setModules([]);
      expect(useAppStore.getState().modules).toEqual([]);
    });
  });

  describe("toggleModule（需求 3.3）", () => {
    it("应将未选中的模块切换为选中", () => {
      useAppStore.getState().setModules(mockModules);
      useAppStore.getState().toggleModule("auth");

      expect(useAppStore.getState().selectedModules.has("auth")).toBe(true);
    });

    it("应将已选中的模块切换为未选中", () => {
      useAppStore.getState().setModules(mockModules);
      useAppStore.getState().toggleModule("auth");
      useAppStore.getState().toggleModule("auth");

      expect(useAppStore.getState().selectedModules.has("auth")).toBe(false);
    });

    it("切换一个模块不应影响其他模块的选中状态", () => {
      useAppStore.getState().setModules(mockModules);
      useAppStore.getState().toggleModule("auth");
      useAppStore.getState().toggleModule("billing");

      const selected = useAppStore.getState().selectedModules;
      expect(selected.has("auth")).toBe(true);
      expect(selected.has("billing")).toBe(true);
      expect(selected.has("reports")).toBe(false);
    });
  });

  describe("selectAll（需求 3.4）", () => {
    it("应选中所有模块", () => {
      useAppStore.getState().setModules(mockModules);
      useAppStore.getState().selectAll();

      const selected = useAppStore.getState().selectedModules;
      expect(selected.size).toBe(3);
      expect(selected.has("auth")).toBe(true);
      expect(selected.has("billing")).toBe(true);
      expect(selected.has("reports")).toBe(true);
    });

    it("模块列表为空时全选结果应为空集合", () => {
      useAppStore.getState().setModules([]);
      useAppStore.getState().selectAll();

      expect(useAppStore.getState().selectedModules.size).toBe(0);
    });

    it("部分已选中时全选应补全所有模块", () => {
      useAppStore.getState().setModules(mockModules);
      useAppStore.getState().toggleModule("auth");
      useAppStore.getState().selectAll();

      expect(useAppStore.getState().selectedModules.size).toBe(3);
    });
  });

  describe("invertSelection（需求 3.5）", () => {
    it("全部未选中时反选应全部选中", () => {
      useAppStore.getState().setModules(mockModules);
      useAppStore.getState().invertSelection();

      const selected = useAppStore.getState().selectedModules;
      expect(selected.size).toBe(3);
    });

    it("全部选中时反选应全部取消", () => {
      useAppStore.getState().setModules(mockModules);
      useAppStore.getState().selectAll();
      useAppStore.getState().invertSelection();

      expect(useAppStore.getState().selectedModules.size).toBe(0);
    });

    it("部分选中时反选应翻转每个模块的状态", () => {
      useAppStore.getState().setModules(mockModules);
      useAppStore.getState().toggleModule("auth");
      // auth=选中, billing=未选中, reports=未选中
      useAppStore.getState().invertSelection();

      const selected = useAppStore.getState().selectedModules;
      // auth=未选中, billing=选中, reports=选中
      expect(selected.has("auth")).toBe(false);
      expect(selected.has("billing")).toBe(true);
      expect(selected.has("reports")).toBe(true);
    });
  });

  describe("setClientName", () => {
    it("应设置客户名称", () => {
      useAppStore.getState().setClientName("客户B");
      expect(useAppStore.getState().clientName).toBe("客户B");
    });
  });

  describe("setBuildingState", () => {
    it("应设置构建中状态", () => {
      useAppStore.getState().setBuildingState(true);
      expect(useAppStore.getState().isBuilding).toBe(true);

      useAppStore.getState().setBuildingState(false);
      expect(useAppStore.getState().isBuilding).toBe(false);
    });
  });

  describe("setBuildResult", () => {
    it("应设置构建结果", () => {
      const result = {
        zip_path: "/output/dist_客户A.zip",
        client_name: "客户A",
        module_count: 2,
      };
      useAppStore.getState().setBuildResult(result);
      expect(useAppStore.getState().buildResult).toEqual(result);
    });

    it("应支持清除构建结果", () => {
      useAppStore.getState().setBuildResult({
        zip_path: "/output/dist.zip",
        client_name: "test",
        module_count: 1,
      });
      useAppStore.getState().setBuildResult(null);
      expect(useAppStore.getState().buildResult).toBeNull();
    });
  });

  describe("reset", () => {
    it("应重置模块和构建状态，但保留项目路径、核心文件和当前页面", () => {
      const store = useAppStore.getState();
      store.setProject("/project", ["main.py", "core/"]);
      store.setModules(mockModules);
      store.toggleModule("auth");
      store.setClientName("客户C");
      store.setBuildingState(true);
      store.setBuildResult({
        zip_path: "/output/dist.zip",
        client_name: "客户C",
        module_count: 1,
      });
      // 设置 V2 状态
      store.setCurrentPage('build');
      store.setCategories([{ id: 1, name: "前端", description: null, created_at: "2024-01-01" }]);
      store.setProjects([{ id: 1, name: "项目A", category_id: 1, repo_path: "/a", tech_stack_type: "vue3", created_at: "2024-01-01", updated_at: "2024-01-01" }]);
      store.setSelectedProjectId(1);
      store.setClients([{ id: 1, name: "客户X", created_at: "2024-01-01" }]);
      store.setBuildRecords([{ id: 1, project_id: 1, client_id: 1, selected_modules: "[]", output_path: "/out", created_at: "2024-01-01" }]);

      useAppStore.getState().reset();

      const state = useAppStore.getState();
      // 项目路径和核心文件应保留
      expect(state.projectPath).toBe("/project");
      expect(state.coreFiles).toEqual(["main.py", "core/"]);
      // 当前页面应保留
      expect(state.currentPage).toBe('build');
      // V2 项目管理状态应被重置
      expect(state.categories).toEqual([]);
      expect(state.projects).toEqual([]);
      expect(state.selectedProjectId).toBeNull();
      expect(state.clients).toEqual([]);
      expect(state.buildRecords).toEqual([]);
      // V1 模块和构建状态应被重置
      expect(state.modules).toEqual([]);
      expect(state.selectedModules.size).toBe(0);
      expect(state.clientName).toBe("");
      expect(state.isBuilding).toBe(false);
      expect(state.buildResult).toBeNull();
    });
  });

  // ========== V2 导航状态测试 ==========

  describe("setCurrentPage", () => {
    it("应设置当前活动页面", () => {
      useAppStore.getState().setCurrentPage('build');
      expect(useAppStore.getState().currentPage).toBe('build');
    });

    it("应支持切换到所有有效页面", () => {
      const pages = ['projects', 'build', 'settings', 'about'] as const;
      for (const page of pages) {
        useAppStore.getState().setCurrentPage(page);
        expect(useAppStore.getState().currentPage).toBe(page);
      }
    });

    it("默认页面应为 projects", () => {
      expect(useAppStore.getState().currentPage).toBe('projects');
    });
  });

  // ========== V2 项目管理状态测试 ==========

  describe("setCategories", () => {
    it("应设置分类列表", () => {
      const categories = [
        { id: 1, name: "前端", description: "前端项目", created_at: "2024-01-01" },
        { id: 2, name: "后端", description: null, created_at: "2024-01-02" },
      ];
      useAppStore.getState().setCategories(categories);
      expect(useAppStore.getState().categories).toEqual(categories);
    });

    it("应支持设置空分类列表", () => {
      useAppStore.getState().setCategories([]);
      expect(useAppStore.getState().categories).toEqual([]);
    });
  });

  describe("setProjects", () => {
    it("应设置项目列表", () => {
      const projects = [
        { id: 1, name: "项目A", category_id: 1, repo_path: "/a", tech_stack_type: "fastapi", created_at: "2024-01-01", updated_at: "2024-01-01" },
      ];
      useAppStore.getState().setProjects(projects);
      expect(useAppStore.getState().projects).toEqual(projects);
    });
  });

  describe("setSelectedProjectId", () => {
    it("应设置选中的项目 ID", () => {
      useAppStore.getState().setSelectedProjectId(42);
      expect(useAppStore.getState().selectedProjectId).toBe(42);
    });

    it("应支持清除选中的项目", () => {
      useAppStore.getState().setSelectedProjectId(42);
      useAppStore.getState().setSelectedProjectId(null);
      expect(useAppStore.getState().selectedProjectId).toBeNull();
    });
  });

  describe("setClients", () => {
    it("应设置客户列表", () => {
      const clients = [
        { id: 1, name: "客户A", created_at: "2024-01-01" },
        { id: 2, name: "客户B", created_at: "2024-01-02" },
      ];
      useAppStore.getState().setClients(clients);
      expect(useAppStore.getState().clients).toEqual(clients);
    });
  });

  describe("setBuildRecords", () => {
    it("应设置构建历史记录列表", () => {
      const records = [
        { id: 1, project_id: 1, client_id: 1, selected_modules: '["auth"]', output_path: "/out/a.zip", created_at: "2024-01-01" },
      ];
      useAppStore.getState().setBuildRecords(records);
      expect(useAppStore.getState().buildRecords).toEqual(records);
    });

    it("应支持设置空记录列表", () => {
      useAppStore.getState().setBuildRecords([]);
      expect(useAppStore.getState().buildRecords).toEqual([]);
    });
  });

  // ========== 属性测试 (Property-Based Tests) ==========

  // Feature: prism-console-v2, Property 15: Page Navigation State
  // **Validates: Requirements 8.2**
  describe("Property 15: 页面导航状态属性测试", () => {
    it("对任意有效页面 ID，setCurrentPage 后 currentPage 应反映该页面 ID", () => {
      fc.assert(
        fc.property(
          fc.constantFrom('projects' as const, 'build' as const, 'settings' as const, 'about' as const),
          (pageId) => {
            // 每次迭代前重置 store 到初始状态
            resetStore();

            // 调用 setCurrentPage 设置页面
            useAppStore.getState().setCurrentPage(pageId);

            // 验证 currentPage 等于设置的页面 ID
            const state = useAppStore.getState();
            expect(state.currentPage).toBe(pageId);
          }
        ),
        { numRuns: 100 }
      );
    });

    it("默认初始页面应为 'projects'", () => {
      fc.assert(
        fc.property(
          fc.constant(null),
          () => {
            // 重置 store 到初始状态
            resetStore();

            // 验证默认页面为 'projects'
            const state = useAppStore.getState();
            expect(state.currentPage).toBe('projects');
          }
        ),
        { numRuns: 100 }
      );
    });
  });
});
