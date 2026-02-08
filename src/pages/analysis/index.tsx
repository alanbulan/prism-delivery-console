/**
 * 项目分析页面 (AnalysisPage) - 容器组件
 *
 * 职责：
 * - Tab 标签切换布局（概览 / 文件分析 / 依赖拓扑 / 语义搜索）
 * - 项目选择 + 全局操作按钮
 * - 通过 useAnalysisData composable 管理所有数据和逻辑
 */

import { useState } from "react";
import {
  Network, Loader2, GitBranch,
  BarChart3, FileText, Radar, FolderTree,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { useAnalysisData } from "./composables/useAnalysisData";
import { OverviewPanel } from "./components/OverviewPanel";
import { FileAnalysisPanel } from "./components/FileAnalysisPanel";
import { TopologyView } from "./components/TopologyView";
import { SemanticSearchPanel } from "./components/SemanticSearchPanel";

/** Tab 标识 */
type AnalysisTab = "overview" | "files" | "topology" | "search";

/** Tab 配置 */
const TABS: { id: AnalysisTab; label: string; icon: React.ReactNode }[] = [
  { id: "overview", label: "概览", icon: <BarChart3 className="h-3.5 w-3.5" /> },
  { id: "files", label: "文件分析", icon: <FileText className="h-3.5 w-3.5" /> },
  { id: "topology", label: "依赖拓扑", icon: <FolderTree className="h-3.5 w-3.5" /> },
  { id: "search", label: "语义搜索", icon: <Radar className="h-3.5 w-3.5" /> },
];

export function AnalysisPage() {
  const {
    projects,
    selectedProjectId,
    fileEntries,
    scanning,
    analyzingFiles,
    batchAnalyzing,
    dependencyGraph,
    analyzingDeps,
    embeddingAll,
    searchResults,
    searching,
    overview,
    loadingOverview,
    handleProjectChange,
    handleScan,
    handleAnalyzeFile,
    handleAnalyzeAll,
    handleAnalyzeDeps,
    handleEmbedAll,
    handleSearch,
    handleGetOverview,
    // 签名索引 + 报告
    indexingSignatures,
    signaturesIndexed,
    report,
    generatingReport,
    handleIndexSignatures,
    handleGenerateReport,
  } = useAnalysisData();

  const [activeTab, setActiveTab] = useState<AnalysisTab>("overview");

  // 空状态：无项目
  if (projects.length === 0) {
    return (
      <div className="flex flex-1 flex-col items-center justify-center text-muted-foreground">
        <Network className="mb-3 h-12 w-12 opacity-30" />
        <p className="text-sm">暂无项目，请先在项目管理页面创建项目</p>
      </div>
    );
  }

  const hasProject = !!selectedProjectId;

  return (
    <div className="flex flex-1 flex-col overflow-hidden">
      {/* 页面标题栏 + 项目选择 */}
      <header className="glass-subtle flex items-center gap-3 px-5 py-3">
        <h2 className="text-base font-semibold text-foreground shrink-0">项目分析</h2>

        <select
          value={selectedProjectId ?? ""}
          onChange={(e) => {
            const val = e.target.value;
            handleProjectChange(val ? Number(val) : null);
          }}
          className="flex-1 max-w-xs rounded-lg border border-border bg-background px-3 py-1.5 text-sm text-foreground outline-none focus:ring-1 focus:ring-ring"
        >
          <option value="">选择项目</option>
          {projects.map((p) => (
            <option key={p.id} value={p.id}>{p.name}</option>
          ))}
        </select>

        {/* 依赖分析按钮已移至依赖拓扑 Tab 内 */}
      </header>

      {/* Tab 栏 */}
      {hasProject && (
        <nav className="flex items-center gap-1 border-b border-border px-5">
          {TABS.map((tab) => (
            <button
              key={tab.id}
              onClick={() => setActiveTab(tab.id)}
              className={`flex items-center gap-1.5 px-3 py-2 text-sm transition-colors border-b-2 -mb-px ${
                activeTab === tab.id
                  ? "border-primary text-primary font-medium"
                  : "border-transparent text-muted-foreground hover:text-foreground"
              }`}
            >
              {tab.icon}
              {tab.label}
            </button>
          ))}
        </nav>
      )}

      {/* Tab 内容区域 — 拓扑 Tab 不加 padding，让图表填满 */}
      <div className={`flex-1 overflow-hidden flex flex-col ${activeTab === "topology" ? "" : "overflow-auto p-5"}`}>
        {!hasProject ? (
          <div className="flex flex-1 items-center justify-center h-full text-muted-foreground">
            <p className="text-sm">请先选择一个项目</p>
          </div>
        ) : activeTab === "overview" ? (
          <OverviewPanel
            overview={overview}
            loading={loadingOverview}
            hasProject={hasProject}
            onLoad={handleGetOverview}
            hasFileIndex={fileEntries.length > 0}
            scanning={scanning}
            onScan={handleScan}
            indexingSignatures={indexingSignatures}
            signaturesIndexed={signaturesIndexed}
            onIndexSignatures={handleIndexSignatures}
            embeddingAll={embeddingAll}
            onEmbedAll={handleEmbedAll}
            report={report}
            generatingReport={generatingReport}
            onGenerateReport={handleGenerateReport}
          />
        ) : activeTab === "files" ? (
          <FileAnalysisPanel
            fileEntries={fileEntries}
            scanning={scanning}
            analyzingFiles={analyzingFiles}
            batchAnalyzing={batchAnalyzing}
            hasProject={hasProject}
            onScan={handleScan}
            onAnalyzeFile={handleAnalyzeFile}
            onAnalyzeAll={handleAnalyzeAll}
          />
        ) : activeTab === "topology" ? (
          dependencyGraph ? (
            <TopologyView graph={dependencyGraph} />
          ) : (
            <div className="flex flex-1 items-center justify-center h-full text-muted-foreground">
              <div className="flex flex-col items-center gap-3">
                <Network className="h-10 w-10 opacity-30" />
                <p className="text-sm">点击下方按钮分析项目依赖关系</p>
                <Button
                  variant="outline"
                  size="sm"
                  onClick={handleAnalyzeDeps}
                  disabled={analyzingDeps}
                  className="gap-2"
                >
                  {analyzingDeps ? (
                    <Loader2 className="h-3.5 w-3.5 animate-spin" />
                  ) : (
                    <GitBranch className="h-3.5 w-3.5" />
                  )}
                  {analyzingDeps ? "分析中..." : "分析依赖"}
                </Button>
              </div>
            </div>
          )
        ) : activeTab === "search" ? (
          <SemanticSearchPanel
            searchResults={searchResults}
            searching={searching}
            embeddingAll={embeddingAll}
            hasFiles={fileEntries.length > 0}
            onSearch={handleSearch}
            onEmbedAll={handleEmbedAll}
          />
        ) : null}
      </div>
    </div>
  );
}
