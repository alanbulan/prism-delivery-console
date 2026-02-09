/**
 * useBuildData - 构建交付页面数据加载与构建逻辑
 *
 * 职责：
 * - 加载项目列表、客户列表、构建历史
 * - 扫描模块
 * - 执行构建操作
 */

import { useEffect, useState, useCallback, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { UnlistenFn } from "@tauri-apps/api/event";
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
  // 项目骨架文件列表（排除模块目录后的核心文件树）
  const [skeletonFiles, setSkeletonFiles] = useState<string[]>([]);

  // ---- 构建日志状态 ----
  const [buildLogs, setBuildLogs] = useState<string[]>([]);
  const [showBuildLog, setShowBuildLog] = useState(false);
  const unlistenRef = useRef<UnlistenFn | null>(null);

  /** 添加一条构建日志（带时间戳） */
  const appendLog = useCallback((msg: string) => {
    const now = new Date();
    const ts = `${String(now.getHours()).padStart(2, "0")}:${String(now.getMinutes()).padStart(2, "0")}:${String(now.getSeconds()).padStart(2, "0")}`;
    setBuildLogs((prev) => [...prev, `[${ts}] ${msg}`]);
  }, []);

  /** 启动构建日志监听 */
  const startLogListener = useCallback(async () => {
    // 先清理旧监听
    if (unlistenRef.current) {
      unlistenRef.current();
      unlistenRef.current = null;
    }
    const unlisten = await listen<string>("build-log", (event) => {
      appendLog(event.payload);
    });
    unlistenRef.current = unlisten;
  }, [appendLog]);

  /** 停止构建日志监听 */
  const stopLogListener = useCallback(() => {
    if (unlistenRef.current) {
      unlistenRef.current();
      unlistenRef.current = null;
    }
  }, []);

  // 组件卸载时清理监听
  useEffect(() => {
    return () => {
      if (unlistenRef.current) {
        unlistenRef.current();
      }
    };
  }, []);

  /** 当前选中的项目对象 */
  const selectedProject = projects.find((p) => p.id === selectedProjectId) ?? null;

  // ---- 数据加载 ----

  /** 加载客户模块配置并自动勾选模块 */
  const loadClientModuleConfig = useCallback(
    async (clientId: number, projectId: number) => {
      try {
        const json = await invoke<string | null>("db_load_client_modules", {
          clientId,
          projectId,
        });
        if (json) {
          const moduleNames: string[] = JSON.parse(json);
          // 将记忆的模块名称设置为选中状态
          const next = new Set(moduleNames.filter((name) => modules.some((m) => m.name === name)));
          useAppStore.setState({ selectedModules: next });
        }
      } catch {
        // 加载失败不阻断流程，静默忽略
      }
    },
    [modules]
  );

  /** 保存客户模块配置（构建成功后调用） */
  const saveClientModuleConfig = useCallback(
    async (clientId: number, projectId: number, moduleNames: string[]) => {
      try {
        await invoke("db_save_client_modules", {
          clientId,
          projectId,
          modulesJson: JSON.stringify(moduleNames),
        });
      } catch {
        // 保存失败不阻断流程，静默忽略
      }
    },
    []
  );

  /** 切换客户时自动加载模块配置 */
  const handleClientChange = useCallback(
    (clientId: number | null) => {
      setSelectedClientId(clientId);
      if (clientId && selectedProjectId && modules.length > 0) {
        loadClientModuleConfig(clientId, selectedProjectId);
      }
    },
    [selectedProjectId, modules, loadClientModuleConfig]
  );

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
          modulesDir: project.modules_dir,
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

  /** 扫描项目骨架文件树（排除模块目录后的核心文件） */
  const scanSkeleton = useCallback(async (project: Project) => {
    try {
      const files = await invoke<string[]>("scan_project_skeleton", {
        projectPath: project.repo_path,
        techStack: project.tech_stack_type,
        modulesDir: project.modules_dir,
      });
      setSkeletonFiles(files);
    } catch {
      setSkeletonFiles([]);
    }
  }, []);

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
    scanSkeleton(project);
    loadClients(selectedProjectId);
    loadBuildRecords(selectedProjectId);
    setSelectedClientId(null);
  }, [selectedProjectId, projects, scanModules, scanSkeleton, loadClients, loadBuildRecords, setClients, setBuildRecords, setModules]);

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

    // 初始化构建日志
    setBuildLogs([]);
    setShowBuildLog(true);
    await startLogListener();
    appendLog("🚀 开始构建交付包...");

    try {
      const result = await invoke<BuildResult>("build_project_package", {
        projectPath: selectedProject.repo_path,
        selectedModules: Array.from(selectedModules),
        clientName: client.name,
        techStack: selectedProject.tech_stack_type,
        modulesDir: selectedProject.modules_dir,
      });

      setBuildResult(result);

      // 获取下一个版本号
      appendLog("→ 获取版本号...");
      const version = await invoke<string>("db_get_next_version", {
        clientId: client.id,
        projectId: selectedProject.id,
      });
      appendLog(`✓ 版本号: ${version}`);

      // 生成变更日志（与上次构建的模块差异）
      appendLog("→ 生成变更日志...");
      let changelog: string | null = null;
      try {
        const lastModulesJson = await invoke<string | null>("db_get_last_build_modules", {
          clientId: client.id,
          projectId: selectedProject.id,
        });
        if (lastModulesJson) {
          const lastModules: string[] = JSON.parse(lastModulesJson);
          // 使用实际打包的完整模块列表（含依赖）进行对比
          const currentModules = result.expanded_modules;
          const added = currentModules.filter((m) => !lastModules.includes(m));
          const removed = lastModules.filter((m) => !currentModules.includes(m));
          const parts: string[] = [];
          if (added.length > 0) parts.push(`新增: ${added.join(", ")}`);
          if (removed.length > 0) parts.push(`移除: ${removed.join(", ")}`);
          changelog = parts.length > 0 ? parts.join("; ") : "无变更";
        }
      } catch {
        // 变更日志生成失败不阻断流程
      }

      // 持久化构建记录（使用实际打包的完整模块列表，含依赖分析自动补充的模块）
      appendLog("→ 保存构建记录...");
      await invoke("db_create_build_record", {
        projectId: selectedProject.id,
        clientId: client.id,
        modulesJson: JSON.stringify(result.expanded_modules),
        outputPath: result.zip_path,
        version,
        changelog,
      });

      await loadBuildRecords(selectedProject.id);

      // 构建成功后保存客户模块配置（记忆选择）
      await saveClientModuleConfig(
        selectedClientId,
        selectedProject.id,
        Array.from(selectedModules)
      );

      toast.success(`构建成功：${result.module_count} 个模块`, {
        action: {
          label: "打开文件夹",
          onClick: () => {
            // 直接传 ZIP 完整路径，explorer /select, 会打开所在目录并选中该文件
            invoke("open_folder", { path: result.zip_path }).catch((err) =>
              toast.error(String(err))
            );
          },
        },
      });
      appendLog(`✅ 构建完成！输出: ${result.zip_path}`);
    } catch (err) {
      appendLog(`❌ 构建失败: ${String(err)}`);
      toast.error(String(err));
    } finally {
      stopLogListener();
      setBuildingState(false);
    }
  };

  /** 打开构建记录的输出文件夹（选中 ZIP 文件） */
  const handleOpenRecordFolder = async (outputPath: string) => {
    try {
      // 直接传 ZIP 完整路径，explorer /select, 会打开所在目录并选中该文件
      await invoke("open_folder", { path: outputPath });
    } catch (err) {
      toast.error(String(err));
    }
  };

  /** 删除单条构建记录 */
  const handleDeleteRecord = async (recordId: number, deleteFiles: boolean = false) => {
    try {
      await invoke("db_delete_build_record", { id: recordId, deleteFiles });
      toast.success(deleteFiles ? "已删除记录和文件" : "已删除构建记录");
      if (selectedProjectId) await loadBuildRecords(selectedProjectId);
    } catch (err) {
      toast.error(String(err));
    }
  };

  /** 清空当前项目的所有构建记录 */
  const handleClearAllRecords = async (deleteFiles: boolean = false) => {
    if (!selectedProjectId) return;
    try {
      const count = await invoke<number>("db_delete_all_build_records", {
        projectId: selectedProjectId,
        deleteFiles,
      });
      toast.success(deleteFiles ? `已清空 ${count} 条记录并删除文件` : `已清空 ${count} 条构建记录`);
      await loadBuildRecords(selectedProjectId);
    } catch (err) {
      toast.error(String(err));
    }
  };

  /** 删除 N 天前的构建记录 */
  const handlePurgeRecords = async (days: number, deleteFiles: boolean = false) => {
    if (!selectedProjectId) return;
    try {
      const count = await invoke<number>("db_delete_build_records_before_days", {
        projectId: selectedProjectId,
        days,
        deleteFiles,
      });
      toast.success(deleteFiles ? `已清洗 ${count} 条记录并删除文件` : `已清洗 ${count} 条 ${days} 天前的记录`);
      await loadBuildRecords(selectedProjectId);
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
    skeletonFiles,
    buildLogs,
    showBuildLog,

    // Actions
    setSelectedProjectId,
    setSelectedClientId: handleClientChange,
    setShowBuildLog,
    toggleModule,
    selectAll,
    invertSelection,
    handleBuild,
    handleOpenRecordFolder,
    handleDeleteRecord,
    handleClearAllRecords,
    handlePurgeRecords,
    getClientName,
    getModuleCount,
    /** 重新加载当前项目的客户列表 */
    reloadClients: () => {
      if (selectedProjectId) loadClients(selectedProjectId);
    },
  };
}
