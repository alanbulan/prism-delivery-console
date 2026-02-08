/**
 * useBuildData - 构建交付页面数据加载与构建逻辑
 *
 * 职责：
 * - 加载项目列表、客户列表、构建历史
 * - 扫描模块
 * - 执行构建操作
 */

import { useEffect, useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { toast } from "sonner";
import { useAppStore } from "@/store";
import type { Project, Client, BuildRecord, BuildResult, ModuleInfo } from "@/types";

export function useBuildData() {
  // ---- 全局 Store ----
  const projects = useAppStore((s) => s.projects);
  const selectedProjectId = useAppStore((s) => s.selectedProjectId);
  const clients = useAppStore((s) => s.clients);
  const buildRecords = useAppStore((s) => s.buildRecords);
  const modules = useAppStore((s) => s.modules);
  const selectedModules = useAppStore((s) => s.selectedModules);
  const isBuilding = useAppStore((s) => s.isBuilding);

  const setProjects = useAppStore((s) => s.setProjects);
  const setSelectedProjectId = useAppStore((s) => s.setSelectedProjectId);
  const setClients = useAppStore((s) => s.setClients);
  const setBuildRecords = useAppStore((s) => s.setBuildRecords);
  const setModules = useAppStore((s) => s.setModules);
  const toggleModule = useAppStore((s) => s.toggleModule);
  const selectAll = useAppStore((s) => s.selectAll);
  const invertSelection = useAppStore((s) => s.invertSelection);
  const setBuildingState = useAppStore((s) => s.setBuildingState);
  const setBuildResult = useAppStore((s) => s.setBuildResult);

  // ---- 本地状态 ----
  const [selectedClientId, setSelectedClientId] = useState<number | null>(null);
  const [scanning, setScanning] = useState(false);

  /** 当前选中的项目对象 */
  const selectedProject = projects.find((p) => p.id === selectedProjectId) ?? null;

  // ---- 数据加载 ----

  const loadProjects = useCallback(async () => {
    try {
      const list = await invoke<Project[]>("db_list_projects");
      setProjects(list);
    } catch (err) {
      toast.error(String(err));
    }
  }, [setProjects]);

  const loadClients = useCallback(
    async (projectId: number) => {
      try {
        const list = await invoke<Client[]>("db_list_clients_by_project", { projectId });
        setClients(list);
      } catch (err) {
        toast.error(String(err));
        setClients([]);
      }
    },
    [setClients]
  );

  const loadBuildRecords = useCallback(
    async (projectId: number) => {
      try {
        const list = await invoke<BuildRecord[]>("db_list_build_records", { projectId });
        setBuildRecords(list);
      } catch (err) {
        toast.error(String(err));
        setBuildRecords([]);
      }
    },
    [setBuildRecords]
  );

  const scanModules = useCallback(
    async (project: Project) => {
      setScanning(true);
      try {
        const list = await invoke<ModuleInfo[]>("scan_project_modules", {
          projectPath: project.repo_path,
          techStack: project.tech_stack_type,
        });
        setModules(list);
      } catch (err) {
        toast.error(String(err));
        setModules([]);
      } finally {
        setScanning(false);
      }
    },
    [setModules]
  );

  /** 页面挂载时加载项目列表 */
  useEffect(() => {
    loadProjects();
  }, [loadProjects]);

  /** 项目选择变化时：扫描模块 + 加载客户 + 加载构建历史 */
  useEffect(() => {
    if (!selectedProjectId) {
      setClients([]);
      setBuildRecords([]);
      setModules([]);
      setSelectedClientId(null);
      return;
    }

    const project = projects.find((p) => p.id === selectedProjectId);
    if (!project) return;

    scanModules(project);
    loadClients(selectedProjectId);
    loadBuildRecords(selectedProjectId);
    setSelectedClientId(null);
  }, [selectedProjectId, projects, scanModules, loadClients, loadBuildRecords, setClients, setBuildRecords, setModules]);

  // ---- 构建操作 ----

  const handleBuild = async () => {
    if (!selectedProject) {
      toast.error("请先选择项目");
      return;
    }
    if (selectedModules.size === 0) {
      toast.error("请至少选择一个模块");
      return;
    }
    if (!selectedClientId) {
      toast.error("请选择客户");
      return;
    }

    const client = clients.find((c) => c.id === selectedClientId);
    if (!client) {
      toast.error("客户信息无效");
      return;
    }

    setBuildingState(true);
    setBuildResult(null);

    try {
      const result = await invoke<BuildResult>("build_project_package", {
        projectPath: selectedProject.repo_path,
        selectedModules: Array.from(selectedModules),
        clientName: client.name,
        techStack: selectedProject.tech_stack_type,
      });

      setBuildResult(result);

      // 持久化构建记录
      await invoke("db_create_build_record", {
        projectId: selectedProject.id,
        clientId: client.id,
        modulesJson: JSON.stringify(Array.from(selectedModules)),
        outputPath: result.zip_path,
      });

      await loadBuildRecords(selectedProject.id);

      toast.success(`构建成功：${result.module_count} 个模块`, {
        action: {
          label: "打开文件夹",
          onClick: () => {
            const dirPath = result.zip_path.replace(/[\\/][^\\/]+$/, "");
            invoke("open_folder", { path: dirPath }).catch((err) =>
              toast.error(String(err))
            );
          },
        },
      });
    } catch (err) {
      toast.error(String(err));
    } finally {
      setBuildingState(false);
    }
  };

  /** 打开构建记录的输出文件夹 */
  const handleOpenRecordFolder = async (outputPath: string) => {
    try {
      const dirPath = outputPath.replace(/[\\/][^\\/]+$/, "");
      await invoke("open_folder", { path: dirPath });
    } catch (err) {
      toast.error(String(err));
    }
  };

  /** 根据客户 ID 查找客户名称 */
  const getClientName = (clientId: number): string => {
    return clients.find((c) => c.id === clientId)?.name ?? `客户#${clientId}`;
  };

  /** 解析构建记录中的模块数量 */
  const getModuleCount = (modulesJson: string): number => {
    try {
      const arr = JSON.parse(modulesJson);
      return Array.isArray(arr) ? arr.length : 0;
    } catch {
      return 0;
    }
  };

  return {
    // 数据
    projects,
    selectedProjectId,
    selectedProject,
    clients,
    buildRecords,
    modules,
    selectedModules,
    isBuilding,
    selectedClientId,
    scanning,

    // Actions
    setSelectedProjectId,
    setSelectedClientId,
    toggleModule,
    selectAll,
    invertSelection,
    handleBuild,
    handleOpenRecordFolder,
    getClientName,
    getModuleCount,
  };
}
