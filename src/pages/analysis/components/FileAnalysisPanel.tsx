/**
 * 文件分析面板 (FileAnalysisPanel)
 *
 * 职责：展示文件列表、状态标识、摘要预览、单个/批量生成操作
 */

import { Loader2, Sparkles, FileText, Search } from "lucide-react";
import { Button } from "@/components/ui/button";
import type { FileIndexEntry } from "@/types";

interface FileAnalysisPanelProps {
  fileEntries: FileIndexEntry[];
  scanning: boolean;
  analyzingFiles: Set<string>;
  batchAnalyzing: boolean;
  hasProject: boolean;
  onScan: () => void;
  onAnalyzeFile: (filePath: string) => void;
  onAnalyzeAll: () => void;
}

export function FileAnalysisPanel({
  fileEntries, scanning, analyzingFiles, batchAnalyzing,
  hasProject, onScan, onAnalyzeFile, onAnalyzeAll,
}: FileAnalysisPanelProps) {
  const changedCount = fileEntries.filter((e) => e.changed || !e.summary).length;

  // 空状态
  if (fileEntries.length === 0) {
    return (
      <div className="flex flex-1 items-center justify-center">
        <div className="flex flex-col items-center gap-3 text-muted-foreground">
          <Search className="h-10 w-10 opacity-30" />
          <p className="text-sm">
            {hasProject ? '点击"扫描文件"开始分析项目' : "请先选择项目"}
          </p>
          {hasProject && (
            <Button onClick={onScan} disabled={scanning} variant="outline" className="gap-2">
              {scanning ? <Loader2 className="h-4 w-4 animate-spin" /> : <Search className="h-4 w-4" />}
              扫描文件
            </Button>
          )}
        </div>
      </div>
    );
  }

  return (
    <div className="flex flex-col gap-3">
      {/* 操作栏 */}
      <div className="flex items-center gap-2">
        <Button onClick={onScan} disabled={scanning} variant="outline" size="sm" className="gap-2">
          {scanning ? <Loader2 className="h-3.5 w-3.5 animate-spin" /> : <Search className="h-3.5 w-3.5" />}
          重新扫描
        </Button>
        <Button onClick={onAnalyzeAll} disabled={batchAnalyzing || changedCount === 0} variant="outline" size="sm" className="gap-2">
          {batchAnalyzing ? <Loader2 className="h-3.5 w-3.5 animate-spin" /> : <Sparkles className="h-3.5 w-3.5" />}
          {batchAnalyzing ? "生成中..." : `全部生成 (${changedCount})`}
        </Button>
      </div>

      {/* 文件列表 */}
      <section className="glass flex flex-col gap-0 overflow-hidden">
        {/* 表头 */}
        <div className="grid grid-cols-[1fr_80px_1fr_90px] gap-2 border-b border-border px-4 py-2.5 text-xs font-medium text-muted-foreground">
          <span>文件路径</span>
          <span className="text-center">状态</span>
          <span>摘要</span>
          <span className="text-center">操作</span>
        </div>

        {/* 文件行 */}
        <div className="max-h-[calc(100vh-320px)] overflow-auto">
          {fileEntries.map((entry) => {
            const isAnalyzing = analyzingFiles.has(entry.relative_path);
            const needsSummary = entry.changed || !entry.summary;

            return (
              <div
                key={entry.relative_path}
                className="grid grid-cols-[1fr_80px_1fr_90px] gap-2 border-b border-border/50 px-4 py-2 text-sm hover:bg-muted/30 transition-colors"
              >
                <div className="flex items-center gap-1.5 min-w-0">
                  <FileText className="h-3.5 w-3.5 shrink-0 text-muted-foreground" />
                  <span className="truncate text-foreground" title={entry.relative_path}>
                    {entry.relative_path}
                  </span>
                </div>

                <div className="flex items-center justify-center">
                  {entry.changed ? (
                    <span className="rounded-full bg-amber-100 px-2 py-0.5 text-xs text-amber-700">已变更</span>
                  ) : entry.summary ? (
                    <span className="rounded-full bg-green-100 px-2 py-0.5 text-xs text-green-700">已分析</span>
                  ) : (
                    <span className="rounded-full bg-gray-100 px-2 py-0.5 text-xs text-gray-500">待分析</span>
                  )}
                </div>

                <div className="flex items-center min-w-0">
                  {entry.summary ? (
                    <span className="truncate text-xs text-muted-foreground" title={entry.summary}>{entry.summary}</span>
                  ) : (
                    <span className="text-xs text-muted-foreground/50">暂无摘要</span>
                  )}
                </div>

                <div className="flex items-center justify-center">
                  {needsSummary && (
                    <Button
                      variant="ghost"
                      size="xs"
                      onClick={() => onAnalyzeFile(entry.relative_path)}
                      disabled={isAnalyzing || batchAnalyzing}
                      className="gap-1"
                    >
                      {isAnalyzing ? <Loader2 className="h-3 w-3 animate-spin" /> : <Sparkles className="h-3 w-3" />}
                      生成
                    </Button>
                  )}
                </div>
              </div>
            );
          })}
        </div>

        {/* 底部统计 */}
        <div className="flex items-center justify-between border-t border-border px-4 py-2 text-xs text-muted-foreground">
          <span>共 {fileEntries.length} 个文件</span>
          <span>
            {fileEntries.filter((e) => e.summary).length} 已分析 /
            {" "}{fileEntries.filter((e) => e.changed).length} 已变更
          </span>
        </div>
      </section>
    </div>
  );
}
