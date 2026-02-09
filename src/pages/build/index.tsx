/**
 * 构建交付页面 (BuildPage) - 容器组件
 *
 * 职责：
 * - 编排选择器、模块列表、构建按钮、构建历史子组件
 * - 通过 useBuildData composable 管理所有数据和逻辑
 *
 * 需求: 9.1, 9.2, 9.3, 9.4, 9.5, 6.1, 6.2, 6.3
 */

import { Package, Loader2, FolderTree, ChevronDown, ChevronRight } from "lucide-react";
import { useState } from "react";
import { Button } from "@/components/ui/button";
import { ModuleCard } from "@/components/ModuleCard";
import { useBuildData } from "./composables/useBuildData";
import { BuildSelector } from "./components/BuildSelector";
import { BuildHistory } from "./components/BuildHistory";
import { BuildLogModal } from "./components/BuildLogModal";

export function BuildPage() {
  const {
    projects,
    selectedProjectId,
    clients,
    modules,
    selectedModules,
    isBuilding,
    selectedClientId,
    scanning,
    skeletonFiles,
    buildRecords,
    buildLogs,
    showBuildLog,
    setSelectedProjectId,
    setSelectedClientId,
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
    reloadClients,
  } = useBuildData();

  // 骨架文件列表折叠状态
  const [skeletonExpanded, setSkeletonExpanded] = useState(false);

  // ---- 空状态：无项目 ----
  if (projects.length === 0) {
    return (
      <div className="flex flex-1 flex-col items-center justify-center text-muted-foreground">
        <Package className="mb-3 h-12 w-12 opacity-30" />
        <p className="text-sm">暂无项目，请先在项目管理页面创建项目</p>
      </div>
    );
  }

  return (
    <div className="flex flex-1 flex-col overflow-hidden">
      {/* 页面标题栏 */}
      <header className="glass-subtle flex items-center px-5 py-3">
        <h2 className="text-base font-semibold text-foreground">构建交付</h2>
      </header>

      {/* 主内容区（可滚动） */}
      <main className="flex flex-1 flex-col gap-4 overflow-auto p-4">
        {/* 选择器区域：项目 + 客户 */}
        <BuildSelector
          projects={projects}
          selectedProjectId={selectedProjectId}
          clients={clients}
          selectedClientId={selectedClientId}
          onProjectChange={setSelectedProjectId}
          onClientChange={setSelectedClientId}
          onClientCreated={reloadClients}
        />

        {/* ---- 模块选择区域 ---- */}
        {selectedProjectId && (
          <section className="glass flex flex-col gap-3 p-4">
            {/* 模块区域标题栏 */}
            <div className="flex items-center justify-between">
              <h3 className="text-sm font-semibold text-foreground">
                模块选择
              </h3>
              <div className="flex items-center gap-3">
                {modules.length > 0 && (
                  <div className="flex gap-1.5">
                    <Button variant="outline" size="xs" onClick={selectAll}>
                      全选
                    </Button>
                    <Button variant="outline" size="xs" onClick={invertSelection}>
                      反选
                    </Button>
                  </div>
                )}
                <span className="text-xs text-muted-foreground">
                  已选 {selectedModules.size}/{modules.length}
                </span>
              </div>
            </div>

            {/* 模块卡片网格 */}
            {scanning ? (
              <div className="flex items-center justify-center py-8 text-muted-foreground">
                <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                <span className="text-sm">正在扫描模块...</span>
              </div>
            ) : modules.length > 0 ? (
              <div className="grid grid-cols-2 gap-2 sm:grid-cols-3 lg:grid-cols-4">
                {modules.map((m) => (
                  <ModuleCard
                    key={m.name}
                    name={m.name}
                    checked={selectedModules.has(m.name)}
                    onToggle={() => toggleModule(m.name)}
                  />
                ))}
              </div>
            ) : (
              <div className="flex items-center justify-center py-8 text-muted-foreground">
                <p className="text-sm">未扫描到模块</p>
              </div>
            )}
          </section>
        )}

        {/* ---- 项目骨架文件（可折叠） ---- */}
        {selectedProjectId && skeletonFiles.length > 0 && (
          <section className="glass flex flex-col gap-2 p-4">
            <button
              type="button"
              onClick={() => setSkeletonExpanded(!skeletonExpanded)}
              className="flex items-center gap-2 text-left"
            >
              {skeletonExpanded ? (
                <ChevronDown className="h-4 w-4 text-muted-foreground" />
              ) : (
                <ChevronRight className="h-4 w-4 text-muted-foreground" />
              )}
              <FolderTree className="h-4 w-4 text-muted-foreground" />
              <h3 className="text-sm font-semibold text-foreground">
                项目骨架
              </h3>
              <span className="text-xs text-muted-foreground">
                （{skeletonFiles.filter((f) => !f.endsWith("/")).length} 个文件，{skeletonFiles.filter((f) => f.endsWith("/")).length} 个目录，构建时自动包含）
              </span>
            </button>
            {skeletonExpanded && (
              <div className="ml-6 max-h-48 overflow-auto rounded-lg border border-border bg-background/50 p-3">
                <ul className="space-y-0.5 font-mono text-xs text-muted-foreground">
                  {skeletonFiles.map((f) => {
                    // 根据路径深度计算缩进层级（每层 1.25rem）
                    const depth = f.replace(/\/$/, "").split("/").length - 1;
                    const isDir = f.endsWith("/");
                    const name = f.replace(/\/$/, "").split("/").pop() ?? f;
                    return (
                      <li
                        key={f}
                        className={isDir ? "text-foreground/70" : ""}
                        style={{ paddingLeft: `${depth * 1.25}rem` }}
                      >
                        {isDir ? `📁 ${name}/` : `📄 ${name}`}
                      </li>
                    );
                  })}
                </ul>
              </div>
            )}
          </section>
        )}

        {/* ---- 构建按钮 ---- */}
        {selectedProjectId && (
          <div className="flex items-center gap-2 px-1">
            <Button
              onClick={handleBuild}
              disabled={isBuilding || selectedModules.size === 0 || !selectedClientId}
              className="gap-2"
            >
              {isBuilding ? (
                <>
                  <Loader2 className="h-4 w-4 animate-spin" />
                  构建中...
                </>
              ) : (
                <>
                  <Package className="h-4 w-4" />
                  构建交付包
                </>
              )}
            </Button>
          </div>
        )}

        {/* ---- 构建日志模态框 ---- */}
        {showBuildLog && (
          <BuildLogModal
            logs={buildLogs}
            isBuilding={isBuilding}
            onClose={() => setShowBuildLog(false)}
          />
        )}

        {/* ---- 构建历史 ---- */}
        {selectedProjectId && (
          <BuildHistory
            records={buildRecords}
            getClientName={getClientName}
            getModuleCount={getModuleCount}
            onOpenFolder={handleOpenRecordFolder}
            onDeleteRecord={handleDeleteRecord}
            onClearAll={handleClearAllRecords}
            onPurge={handlePurgeRecords}
          />
        )}
      </main>
    </div>
  );
}
