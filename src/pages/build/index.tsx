/**
 * 构建交付页面 (BuildPage) - 容器组件
 *
 * 职责：
 * - 编排选择器、模块列表、构建按钮、构建历史子组件
 * - 通过 useBuildData composable 管理所有数据和逻辑
 *
 * 需求: 9.1, 9.2, 9.3, 9.4, 9.5, 6.1, 6.2, 6.3
 */

import { Package, Loader2 } from "lucide-react";
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
