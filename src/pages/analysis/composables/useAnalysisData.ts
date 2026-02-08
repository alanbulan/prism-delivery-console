/**
 * 项目分析数据逻辑 composable
 *
 * 职责：
 * - 管理文件索引扫描状态
 * - 管理 LLM 摘要生成状态
 * - 管理签名索引 + 报告生成状态
 * - 封装 Tauri command 调用
 */

import { useState, useCallback, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { toast } from "sonner";
import type {
  Project,
  FileIndexEntry,
  DependencyGraph,
  SimilarFile,
  EmbedBatchResult,
  ProjectOverview,
  IndexSignaturesResult,
} from "@/types";
import { useAppStore } from "@/store";

export function useAnalysisData() {
  const projects = useAppStore((s) => s.projects);

  // ---- 基础状态 ----
  const [selectedProjectId, setSelectedProjectId] = useState<number | null>(null);
  const [fileEntries, setFileEntries] = useState<FileIndexEntry[]>([]);
  const [scanning, setScanning] = useState(false);
  const [analyzingFiles, setAnalyzingFiles] = useState<Set<string>>(new Set());
  const [batchAnalyzing, setBatchAnalyzing] = useState(false);
  const [dependencyGraph, setDependencyGraph] = useState<DependencyGraph | null>(null);
  const [analyzingDeps, setAnalyzingDeps] = useState(false);
  const [embeddingAll, setEmbeddingAll] = useState(false);
  const [searchQuery, setSearchQuery] = useState("");
  const [searchResults, setSearchResults] = useState<SimilarFile[]>([]);
  const [searching, setSearching] = useState(false);
  const [overview, setOverview] = useState<ProjectOverview | null>(null);
  const [loadingOverview, setLoadingOverview] = useState(false);

  // ---- 签名索引 + 报告状态 ----
  /** 是否正在提取签名 */
  const [indexingSignatures, setIndexingSignatures] = useState(false);
  /** 签名是否已索引 */
  const [signaturesIndexed, setSigIndexed] = useState(false);
  /** AI 生成的项目报告（Markdown） */
  const [report, setReport] = useState<string | null>(null);
  /** 是否正在生成报告 */
  const [generatingReport, setGeneratingReport] = useState(false);

  /** 获取当前选中的项目对象 */
  const selectedProject: Project | undefined = projects.find(
    (p) => p.id === selectedProjectId
  );

  /** 扫描项目文件索引 */
  const handleScan = useCallback(async () => {
    if (!selectedProjectId || !selectedProject) return;
    setScanning(true);
    try {
      const entries = await invoke<FileIndexEntry[]>("scan_project_file_index", {
        projectId: selectedProjectId,
        projectPath: selectedProject.repo_path,
      });
      setFileEntries(entries);
      const changedCount = entries.filter((e) => e.changed).length;
      toast.success(`扫描完成：${entries.length} 个文件，${changedCount} 个有变更`);
    } catch (err) {
      toast.error(`扫描失败：${String(err)}`);
    } finally {
      setScanning(false);
    }
  }, [selectedProjectId, selectedProject]);

  /** 为单个文件生成 LLM 摘要 */
  const handleAnalyzeFile = useCallback(
    async (filePath: string) => {
      if (!selectedProjectId || !selectedProject) return;
      setAnalyzingFiles((prev) => new Set(prev).add(filePath));
      try {
        const summary = await invoke<string>("analyze_file_summary", {
          projectId: selectedProjectId,
          projectPath: selectedProject.repo_path,
          filePath,
        });
        setFileEntries((prev) =>
          prev.map((e) =>
            e.relative_path === filePath
              ? { ...e, summary, changed: false }
              : e
          )
        );
      } catch (err) {
        toast.error(`生成摘要失败 (${filePath})：${String(err)}`);
      } finally {
        setAnalyzingFiles((prev) => {
          const next = new Set(prev);
          next.delete(filePath);
          return next;
        });
      }
    },
    [selectedProjectId, selectedProject]
  );

  /** 批量生成所有变更文件的摘要 */
  const handleAnalyzeAll = useCallback(async () => {
    if (!selectedProjectId || !selectedProject) return;
    const changedFiles = fileEntries.filter((e) => e.changed || !e.summary);
    if (changedFiles.length === 0) {
      toast.info("没有需要生成摘要的文件");
      return;
    }
    setBatchAnalyzing(true);
    let successCount = 0;
    let failCount = 0;
    for (const file of changedFiles) {
      try {
        const summary = await invoke<string>("analyze_file_summary", {
          projectId: selectedProjectId,
          projectPath: selectedProject.repo_path,
          filePath: file.relative_path,
        });
        setFileEntries((prev) =>
          prev.map((e) =>
            e.relative_path === file.relative_path
              ? { ...e, summary, changed: false }
              : e
          )
        );
        successCount++;
      } catch {
        failCount++;
      }
    }
    setBatchAnalyzing(false);
    if (failCount > 0) {
      toast.warning(`批量生成完成：${successCount} 成功，${failCount} 失败`);
    } else {
      toast.success(`批量生成完成：${successCount} 个文件`);
    }
  }, [selectedProjectId, selectedProject, fileEntries]);

  /** 分析项目文件依赖关系 */
  const handleAnalyzeDeps = useCallback(async () => {
    if (!selectedProject) return;
    setAnalyzingDeps(true);
    try {
      const graph = await invoke<DependencyGraph>("analyze_dependencies", {
        projectPath: selectedProject.repo_path,
      });
      setDependencyGraph(graph);
      toast.success(`依赖分析完成：${graph.nodes.length} 个节点，${graph.edges.length} 条依赖`);
    } catch (err) {
      toast.error(`依赖分析失败：${String(err)}`);
    } finally {
      setAnalyzingDeps(false);
    }
  }, [selectedProject]);

  /** 切换项目时清空所有状态 */
  const handleProjectChange = useCallback((id: number | null) => {
    setSelectedProjectId(id);
    setFileEntries([]);
    setDependencyGraph(null);
    setSearchResults([]);
    setOverview(null);
    setSearchQuery("");
    setReport(null);
    setSigIndexed(false);
  }, []);

  /** 批量生成所有文件的 Embedding */
  const handleEmbedAll = useCallback(async () => {
    if (!selectedProjectId || !selectedProject) return;
    setEmbeddingAll(true);
    try {
      const result = await invoke<EmbedBatchResult>("embed_all_files", {
        projectId: selectedProjectId,
        projectPath: selectedProject.repo_path,
      });
      if (result.failed > 0) {
        toast.warning(`Embedding 生成完成：${result.success} 成功，${result.failed} 失败`);
      } else if (result.success === 0) {
        toast.info("所有文件已有 Embedding，无需重新生成");
      } else {
        toast.success(`Embedding 生成完成：${result.success} 个文件`);
      }
    } catch (err) {
      toast.error(`Embedding 生成失败：${String(err)}`);
    } finally {
      setEmbeddingAll(false);
    }
  }, [selectedProjectId, selectedProject]);

  /** 获取项目概览 */
  const handleGetOverview = useCallback(async () => {
    if (!selectedProject) return;
    setLoadingOverview(true);
    try {
      const data = await invoke<ProjectOverview>("get_project_overview", {
        projectPath: selectedProject.repo_path,
      });
      setOverview(data);
      toast.success(`概览加载完成：${data.total_files} 个文件，${data.total_lines} 行代码`);
    } catch (err) {
      toast.error(`获取概览失败：${String(err)}`);
    } finally {
      setLoadingOverview(false);
    }
  }, [selectedProject]);

  /** 语义搜索 */
  const handleSearch = useCallback(async (query: string) => {
    if (!selectedProjectId || !query.trim()) return;
    setSearching(true);
    setSearchQuery(query);
    try {
      const results = await invoke<SimilarFile[]>("search_similar_files", {
        projectId: selectedProjectId,
        query: query.trim(),
        topK: 10,
      });
      setSearchResults(results);
      if (results.length === 0) {
        toast.info("未找到相似文件，请先生成 Embedding");
      }
    } catch (err) {
      toast.error(`搜索失败：${String(err)}`);
    } finally {
      setSearching(false);
    }
  }, [selectedProjectId]);

  /** 提取项目签名索引 */
  const handleIndexSignatures = useCallback(async () => {
    if (!selectedProjectId || !selectedProject) return;
    setIndexingSignatures(true);
    try {
      const result = await invoke<IndexSignaturesResult>("index_project_signatures", {
        projectId: selectedProjectId,
        projectPath: selectedProject.repo_path,
      });
      setSigIndexed(true);
      toast.success(`签名提取完成：${result.total} 个文件，${result.indexed} 个已索引`);
    } catch (err) {
      toast.error(`签名提取失败：${String(err)}`);
    } finally {
      setIndexingSignatures(false);
    }
  }, [selectedProjectId, selectedProject]);

  /** 生成项目分析报告 */
  const handleGenerateReport = useCallback(async (mode: "fast" | "deep") => {
    if (!selectedProjectId || !selectedProject) return;
    setGeneratingReport(true);
    setReport(null);
    try {
      const result = await invoke<string>("generate_project_report", {
        projectId: selectedProjectId,
        projectPath: selectedProject.repo_path,
        mode,
      });
      setReport(result);
      toast.success("项目报告生成完成");
    } catch (err) {
      toast.error(`报告生成失败：${String(err)}`);
    } finally {
      setGeneratingReport(false);
    }
  }, [selectedProjectId, selectedProject]);

  // ---- 自动后台索引 ----
  /** 用于防止重复触发的 ref */
  const autoIndexTriggered = useRef<number | null>(null);

  /**
   * 当选择项目时，检查 auto_index_signatures 设置。
   * 若开启，自动执行：扫描 → 签名提取（串行，不阻塞 UI）。
   */
  useEffect(() => {
    if (!selectedProjectId || !selectedProject) return;
    // 同一项目只触发一次
    if (autoIndexTriggered.current === selectedProjectId) return;

    let cancelled = false;

    async function autoIndex() {
      try {
        // 读取设置
        const val = await invoke<string | null>("get_app_setting", { key: "auto_index_signatures" });
        if (val !== "true" || cancelled) return;

        // 标记已触发
        autoIndexTriggered.current = selectedProjectId;

        // 步骤 1：扫描文件索引
        const entries = await invoke<FileIndexEntry[]>("scan_project_file_index", {
          projectId: selectedProjectId,
          projectPath: selectedProject!.repo_path,
        });
        if (cancelled) return;
        setFileEntries(entries);

        // 步骤 2：提取签名
        const result = await invoke<IndexSignaturesResult>("index_project_signatures", {
          projectId: selectedProjectId,
          projectPath: selectedProject!.repo_path,
        });
        if (cancelled) return;
        setSigIndexed(true);
        toast.success(`自动索引完成：${result.total} 文件，${result.indexed} 已索引`);
      } catch {
        // 自动索引失败不阻塞用户操作，静默处理
      }
    }

    autoIndex();
    return () => { cancelled = true; };
  }, [selectedProjectId, selectedProject]);

  return {
    projects,
    selectedProjectId,
    selectedProject,
    fileEntries,
    scanning,
    analyzingFiles,
    batchAnalyzing,
    dependencyGraph,
    analyzingDeps,
    embeddingAll,
    searchQuery,
    searchResults,
    searching,
    handleProjectChange,
    handleScan,
    handleAnalyzeFile,
    handleAnalyzeAll,
    handleAnalyzeDeps,
    handleEmbedAll,
    handleSearch,
    overview,
    loadingOverview,
    handleGetOverview,
    // 签名索引 + 报告
    indexingSignatures,
    signaturesIndexed,
    report,
    generatingReport,
    handleIndexSignatures,
    handleGenerateReport,
  };
}
