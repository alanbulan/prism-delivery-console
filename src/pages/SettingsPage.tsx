/**
 * 设置页面 (SettingsPage)
 *
 * 布局结构：
 * ┌──────────────────────────────────────────────────┐
 * │ 设置                                              │
 * ├──────────────────────────────────────────────────┤
 * │ glass card:                                       │
 * │                                                   │
 * │ 默认构建输出目录                                   │
 * │ [/path/to/output/dir          ] [选择文件夹]      │
 * │                                                   │
 * │ 数据库文件路径                                     │
 * │ [/path/to/db/file] (只读)                         │
 * │                                                   │
 * └──────────────────────────────────────────────────┘
 *
 * 职责：
 * - 页面打开时加载当前设置 (get_app_settings)
 * - 默认构建输出目录：支持文件夹选择器，选择后立即持久化
 * - 数据库文件路径：只读展示
 *
 * 需求: 10.1, 10.2, 10.3, 10.4
 */

import { useEffect, useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { toast } from "sonner";
import { FolderOpen, Database, Loader2, Brain, RefreshCw, Eye, EyeOff, ScanSearch, RotateCcw } from "lucide-react";
import { Checkbox } from "@/components/ui/checkbox";
import { Button } from "@/components/ui/button";
import type { AppSettings, LlmConfig, LlmModel } from "@/types";

export function SettingsPage() {
  // ---- 本地状态 ----
  /** 默认构建输出目录 */
  const [outputDir, setOutputDir] = useState<string | null>(null);
  /** 数据库文件路径 */
  const [dbPath, setDbPath] = useState("");
  /** 是否正在加载设置 */
  const [loading, setLoading] = useState(true);

  // ---- LLM 配置状态 ----
  /** LLM API 基础地址 */
  const [llmBaseUrl, setLlmBaseUrl] = useState("");
  /** LLM API Key */
  const [llmApiKey, setLlmApiKey] = useState("");
  /** 当前选中的模型名称 */
  const [llmModelName, setLlmModelName] = useState("");
  /** 当前选中的 Embedding 模型名称 */
  const [llmEmbeddingModel, setLlmEmbeddingModel] = useState("");
  /** 可用模型列表 */
  const [llmModels, setLlmModels] = useState<LlmModel[]>([]);
  /** 是否正在获取模型列表 */
  const [fetchingModels, setFetchingModels] = useState(false);
  /** 是否显示 API Key */
  const [showApiKey, setShowApiKey] = useState(false);

  // ---- 项目分析配置 ----
  /** 自动后台索引开关 */
  const [autoIndexSignatures, setAutoIndexSignatures] = useState(false);

  // ---- 加载设置 ----

  /** 页面挂载时加载当前设置 */
  useEffect(() => {
    async function loadSettings() {
      try {
        const settings = await invoke<AppSettings>("get_app_settings");
        setOutputDir(settings.default_output_dir);
        setDbPath(settings.db_path);

        // 加载 LLM 配置（各字段独立存储在 settings 表中）
        try {
          const config = await invoke<LlmConfig>("get_llm_config");
          setLlmBaseUrl(config.base_url);
          setLlmApiKey(config.api_key);
          setLlmModelName(config.model_name);
          setLlmEmbeddingModel(config.embedding_model);
        } catch {
          // LLM 配置尚未设置，使用默认空值
        }

        // 加载项目分析配置
        try {
          const autoIdx = await invoke<string | null>("get_app_setting", { key: "auto_index_signatures" });
          setAutoIndexSignatures(autoIdx === "true");
        } catch {
          // 尚未设置，使用默认值 false
        }
      } catch (err) {
        toast.error(String(err));
      } finally {
        setLoading(false);
      }
    }
    loadSettings();
  }, []);

  // ---- 操作 ----

  /** 打开文件夹选择器，选择默认构建输出目录并立即持久化 */
  const handlePickOutputDir = async () => {
    try {
      const selected = await open({ directory: true });
      // 用户取消选择时 selected 为 null
      if (!selected) return;

      // 立即持久化到数据库
      await invoke("save_app_setting", {
        key: "default_output_dir",
        value: selected,
      });
      setOutputDir(selected);
      toast.success("默认构建输出目录已更新");
    } catch (err) {
      toast.error(String(err));
    }
  };

  /** 保存单个 LLM 配置项到 settings 表 */
  const saveLlmField = useCallback(async (key: string, value: string) => {
    try {
      await invoke("save_app_setting", { key, value });
    } catch (err) {
      toast.error(`保存 LLM 配置失败：${String(err)}`);
    }
  }, []);

  /** Base URL 失焦时保存 */
  const handleBaseUrlBlur = useCallback(() => {
    if (llmBaseUrl.trim()) {
      saveLlmField("llm_base_url", llmBaseUrl.trim());
      toast.success("API 基础地址已保存");
    }
  }, [llmBaseUrl, saveLlmField]);

  /** API Key 失焦时保存 */
  const handleApiKeyBlur = useCallback(() => {
    if (llmApiKey.trim()) {
      saveLlmField("llm_api_key", llmApiKey.trim());
      toast.success("API Key 已保存");
    }
  }, [llmApiKey, saveLlmField]);

  /** 从 API 获取可用模型列表 */
  const handleFetchModels = useCallback(async () => {
    if (!llmBaseUrl.trim()) {
      toast.error("请先填写 API 基础地址");
      return;
    }
    setFetchingModels(true);
    try {
      const models = await invoke<LlmModel[]>("list_llm_models", {
        baseUrl: llmBaseUrl.trim(),
        apiKey: llmApiKey.trim(),
      });
      setLlmModels(models);
      if (models.length === 0) {
        toast.info("未获取到可用模型");
      } else {
        toast.success(`获取到 ${models.length} 个模型`);
      }
    } catch (err) {
      toast.error(`获取模型列表失败：${String(err)}`);
    } finally {
      setFetchingModels(false);
    }
  }, [llmBaseUrl, llmApiKey]);

  /** 选择模型后立即保存 */
  const handleModelChange = useCallback((e: React.ChangeEvent<HTMLSelectElement>) => {
    const value = e.target.value;
    setLlmModelName(value);
    if (value) {
      saveLlmField("llm_model_name", value);
      toast.success("Chat 模型已保存");
    }
  }, [saveLlmField]);

  /** 选择 Embedding 模型后立即保存 */
  const handleEmbeddingModelChange = useCallback((e: React.ChangeEvent<HTMLSelectElement>) => {
    const value = e.target.value;
    setLlmEmbeddingModel(value);
    if (value) {
      saveLlmField("llm_embedding_model", value);
      toast.success("Embedding 模型已保存");
    }
  }, [saveLlmField]);

  /** 切换自动后台索引开关 */
  const handleAutoIndexToggle = useCallback(async (checked: boolean) => {
    setAutoIndexSignatures(checked);
    try {
      await invoke("save_app_setting", { key: "auto_index_signatures", value: String(checked) });
      toast.success(checked ? "自动后台索引已开启" : "自动后台索引已关闭");
    } catch (err) {
      toast.error(`保存设置失败：${String(err)}`);
    }
  }, []);

  /** 重置 LLM 配置为默认值 */
  const handleResetLlmConfig = useCallback(async () => {
    const keys = ["llm_base_url", "llm_api_key", "llm_model_name", "llm_embedding_model"];
    try {
      for (const key of keys) {
        await invoke("save_app_setting", { key, value: "" });
      }
      setLlmBaseUrl("");
      setLlmApiKey("");
      setLlmModelName("");
      setLlmEmbeddingModel("");
      setLlmModels([]);
      toast.success("LLM 配置已重置");
    } catch (err) {
      toast.error(`重置失败：${String(err)}`);
    }
  }, []);

  // ---- 加载中状态 ----
  if (loading) {
    return (
      <div className="flex flex-1 items-center justify-center text-muted-foreground">
        <Loader2 className="mr-2 h-4 w-4 animate-spin" />
        <span className="text-sm">加载设置中...</span>
      </div>
    );
  }

  // ---- 渲染 ----
  return (
    <div className="flex flex-1 flex-col overflow-hidden">
      {/* 页面标题栏 */}
      <header className="glass-subtle flex items-center px-5 py-3">
        <h2 className="text-base font-semibold text-foreground">设置</h2>
      </header>

      {/* 主内容区 */}
      <main className="flex flex-1 flex-col gap-4 overflow-auto p-4">
        {/* 设置卡片 */}
        <section className="glass flex flex-col gap-6 p-5">
          {/* 默认构建输出目录 */}
          <div className="flex flex-col gap-2">
            <label className="text-sm font-medium text-foreground">
              默认构建输出目录
            </label>
            <div className="flex items-center gap-2">
              <input
                type="text"
                readOnly
                value={outputDir ?? ""}
                placeholder="未设置（将使用系统默认位置）"
                className="flex-1 rounded-lg border border-border bg-background px-3 py-2 text-sm text-foreground outline-none"
              />
              <Button
                variant="outline"
                size="default"
                onClick={handlePickOutputDir}
                className="gap-2 shrink-0"
              >
                <FolderOpen className="h-4 w-4" />
                选择文件夹
              </Button>
            </div>
            <p className="text-xs text-muted-foreground">
              构建交付包时的默认输出位置
            </p>
          </div>

          {/* 数据库文件路径（只读） */}
          <div className="flex flex-col gap-2">
            <div className="flex items-center gap-2">
              <Database className="h-4 w-4 text-muted-foreground" />
              <label className="text-sm font-medium text-foreground">
                数据库文件路径
              </label>
            </div>
            <input
              type="text"
              readOnly
              value={dbPath}
              className="rounded-lg border border-border bg-muted/30 px-3 py-2 text-sm text-muted-foreground outline-none cursor-default"
            />
            <p className="text-xs text-muted-foreground">
              SQLite 数据库存储位置（只读）
            </p>
          </div>
        </section>

        {/* LLM 配置卡片 */}
        <section className="glass flex flex-col gap-6 p-5">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <Brain className="h-4 w-4 text-muted-foreground" />
              <h3 className="text-sm font-semibold text-foreground">
                LLM 模型配置
              </h3>
            </div>
            <Button
              variant="ghost"
              size="sm"
              onClick={handleResetLlmConfig}
              className="gap-1.5 text-muted-foreground hover:text-destructive"
            >
              <RotateCcw className="h-3.5 w-3.5" />
              重置
            </Button>
          </div>
          <p className="text-xs text-muted-foreground -mt-4">
            配置 OpenAI 兼容的 API 接口，用于项目分析功能
          </p>

          {/* API 基础地址 */}
          <div className="flex flex-col gap-2">
            <label className="text-sm font-medium text-foreground">
              API 基础地址
            </label>
            <input
              type="text"
              value={llmBaseUrl}
              onChange={(e) => setLlmBaseUrl(e.target.value)}
              onBlur={handleBaseUrlBlur}
              placeholder="http://localhost:11434/v1"
              className="rounded-lg border border-border bg-background px-3 py-2 text-sm text-foreground outline-none focus:ring-1 focus:ring-ring"
            />
            <p className="text-xs text-muted-foreground">
              支持 OpenAI 兼容接口（如 Ollama、vLLM、LM Studio 等）
            </p>
          </div>

          {/* API Key */}
          <div className="flex flex-col gap-2">
            <label className="text-sm font-medium text-foreground">
              API Key
            </label>
            <div className="flex items-center gap-2">
              <input
                type={showApiKey ? "text" : "password"}
                value={llmApiKey}
                onChange={(e) => setLlmApiKey(e.target.value)}
                onBlur={handleApiKeyBlur}
                placeholder="sk-... （本地模型可留空）"
                className="flex-1 rounded-lg border border-border bg-background px-3 py-2 text-sm text-foreground outline-none focus:ring-1 focus:ring-ring"
              />
              <Button
                variant="ghost"
                size="icon"
                onClick={() => setShowApiKey(!showApiKey)}
                className="shrink-0 h-9 w-9"
                title={showApiKey ? "隐藏" : "显示"}
              >
                {showApiKey ? (
                  <EyeOff className="h-4 w-4" />
                ) : (
                  <Eye className="h-4 w-4" />
                )}
              </Button>
            </div>
          </div>

          {/* 模型选择 */}
          <div className="flex flex-col gap-2">
            <label className="text-sm font-medium text-foreground">
              Chat 模型
            </label>
            <div className="flex items-center gap-2">
              <select
                value={llmModelName}
                onChange={handleModelChange}
                className="flex-1 rounded-lg border border-border bg-background px-3 py-2 text-sm text-foreground outline-none focus:ring-1 focus:ring-ring"
              >
                <option value="">请先获取模型列表</option>
                {llmModels.map((m) => (
                  <option key={m.id} value={m.id}>
                    {m.id}
                  </option>
                ))}
              </select>
              <Button
                variant="outline"
                size="default"
                onClick={handleFetchModels}
                disabled={fetchingModels || !llmBaseUrl.trim()}
                className="gap-2 shrink-0"
              >
                <RefreshCw className={`h-4 w-4 ${fetchingModels ? "animate-spin" : ""}`} />
                获取模型
              </Button>
            </div>
            <p className="text-xs text-muted-foreground">
              用于文件摘要生成的 Chat 模型
            </p>
          </div>

          {/* Embedding 模型选择 */}
          <div className="flex flex-col gap-2">
            <label className="text-sm font-medium text-foreground">
              Embedding 模型
            </label>
            <select
              value={llmEmbeddingModel}
              onChange={handleEmbeddingModelChange}
              className="rounded-lg border border-border bg-background px-3 py-2 text-sm text-foreground outline-none focus:ring-1 focus:ring-ring"
            >
              <option value="">请先获取模型列表</option>
              {llmModels.map((m) => (
                <option key={m.id} value={m.id}>
                  {m.id}
                </option>
              ))}
            </select>
            <p className="text-xs text-muted-foreground">
              用于语义搜索的 Embedding 模型（如 nomic-embed-text、bge-m3）
            </p>
          </div>
        </section>

        {/* 项目分析配置卡片 */}
        <section className="glass flex flex-col gap-6 p-5">
          <div className="flex items-center gap-2">
            <ScanSearch className="h-4 w-4 text-muted-foreground" />
            <h3 className="text-sm font-semibold text-foreground">
              项目分析
            </h3>
          </div>

          {/* 自动后台索引 */}
          <div className="flex items-start gap-3">
            <Checkbox
              id="auto-index"
              checked={autoIndexSignatures}
              onCheckedChange={(checked) => handleAutoIndexToggle(checked === true)}
              className="mt-0.5"
            />
            <div className="flex flex-col gap-1">
              <label htmlFor="auto-index" className="text-sm font-medium text-foreground cursor-pointer">
                自动后台索引
              </label>
              <p className="text-xs text-muted-foreground">
                选择项目时自动执行文件扫描和签名提取，无需手动操作。索引过程在后台运行，不阻塞界面。
              </p>
            </div>
          </div>
        </section>
      </main>
    </div>
  );
}
