import { create } from "zustand";
import type { ModuleInfo, BuildResult, Category, Project, Client, BuildRecord, PageId } from "./types";

// ============================================================
// Slice 类型定义 - 按职责拆分状态
// ============================================================

/** 导航状态 Slice */
interface NavigationSlice {
  /** 当前活动页面标识 */
  currentPage: PageId;
  /** 侧边栏是否收起 */
  sidebarCollapsed: boolean;
  /** 设置当前活动页面 */
  setCurrentPage: (page: PageId) => void;
  /** 切换侧边栏收展状态 */
  toggleSidebar: () => void;
}

/** V2 项目管理状态 Slice */
interface ProjectManagementSlice {
  /** 项目分类列表 */
  categories: Category[];
  /** 项目列表 */
  projects: Project[];
  /** 当前选中的项目 ID */
  selectedProjectId: number | null;
  /** 客户列表 */
  clients: Client[];
  /** 构建历史记录列表 */
  buildRecords: BuildRecord[];

  /** 设置分类列表 */
  setCategories: (categories: Category[]) => void;
  /** 设置项目列表 */
  setProjects: (projects: Project[]) => void;
  /** 设置当前选中的项目 ID */
  setSelectedProjectId: (id: number | null) => void;
  /** 设置客户列表 */
  setClients: (clients: Client[]) => void;
  /** 设置构建历史记录列表 */
  setBuildRecords: (records: BuildRecord[]) => void;
  /** 重置 V2 项目管理相关状态 */
  resetProjectManagement: () => void;
}

/** 模块选择状态 Slice */
interface ModuleSlice {
  /** 扫描到的所有业务模块 */
  modules: ModuleInfo[];
  /** 当前选中的模块名称集合，使用 Set 实现 O(1) 查找 */
  selectedModules: Set<string>;
  /** 是否正在构建中 */
  isBuilding: boolean;
  /** 最近一次构建结果 */
  buildResult: BuildResult | null;

  /** 设置扫描到的模块列表 */
  setModules: (modules: ModuleInfo[]) => void;
  /** 切换单个模块的选中状态 */
  toggleModule: (name: string) => void;
  /** 全选所有模块 */
  selectAll: () => void;
  /** 反选所有模块 */
  invertSelection: () => void;
  /** 设置构建中状态 */
  setBuildingState: (building: boolean) => void;
  /** 设置构建结果 */
  setBuildResult: (result: BuildResult | null) => void;
}

/** 应用全局状态接口 - 由所有 Slice 组合而成 */
type AppStore = NavigationSlice & ProjectManagementSlice & ModuleSlice & {
  /** 重置项目管理和构建相关状态（保留当前页面） */
  reset: () => void;
};

// ============================================================
// Store 实现
// ============================================================

/**
 * 全局状态 Store
 * 使用 Zustand 管理应用状态，按 Slice 模式组织
 */
export const useAppStore = create<AppStore>((set, get) => ({
  // ========== Navigation Slice ==========
  currentPage: 'build',
  sidebarCollapsed: false,
  setCurrentPage: (page) => set({ currentPage: page }),
  toggleSidebar: () => set((state) => ({ sidebarCollapsed: !state.sidebarCollapsed })),

  // ========== Project Management Slice (V2) ==========
  categories: [],
  projects: [],
  selectedProjectId: null,
  clients: [],
  buildRecords: [],

  setCategories: (categories) => set({ categories }),
  setProjects: (projects) => set({ projects }),
  setSelectedProjectId: (id) => set({ selectedProjectId: id }),
  setClients: (clients) => set({ clients }),
  setBuildRecords: (records) => set({ buildRecords: records }),
  resetProjectManagement: () => set({
    categories: [],
    projects: [],
    selectedProjectId: null,
    clients: [],
    buildRecords: [],
  }),

  // ========== Module Slice ==========
  modules: [],
  selectedModules: new Set<string>(),
  isBuilding: false,
  buildResult: null,

  setModules: (modules) => set({ modules }),

  /** 切换模块选中状态：选中则取消，未选中则选中 */
  toggleModule: (name) => set((state) => {
    const next = new Set(state.selectedModules);
    if (next.has(name)) {
      next.delete(name);
    } else {
      next.add(name);
    }
    return { selectedModules: next };
  }),

  /** 全选：将所有模块加入选中集合 */
  selectAll: () => set((state) => ({
    selectedModules: new Set(state.modules.map((m) => m.name)),
  })),

  /** 反选：翻转每个模块的选中状态 */
  invertSelection: () => set((state) => {
    const next = new Set<string>();
    for (const m of state.modules) {
      if (!state.selectedModules.has(m.name)) {
        next.add(m.name);
      }
    }
    return { selectedModules: next };
  }),

  setBuildingState: (building) => set({ isBuilding: building }),
  setBuildResult: (result) => set({ buildResult: result }),

  /** 重置项目管理和构建相关状态，保留当前页面 */
  reset: () => set({
    categories: [],
    projects: [],
    selectedProjectId: null,
    clients: [],
    buildRecords: [],
    modules: [],
    selectedModules: new Set<string>(),
    isBuilding: false,
    buildResult: null,
  }),
}));
