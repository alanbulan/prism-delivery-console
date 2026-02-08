/**
 * 语义搜索面板 (SemanticSearchPanel)
 *
 * 职责：Embedding 生成 + 自然语言语义搜索
 * 布局与其他 Tab 保持一致：空状态居中显示图标+描述+操作按钮
 */

import { useState } from "react";
import { Loader2, Search, Database, FileText, Radar } from "lucide-react";
import { Button } from "@/components/ui/button";
import type { SimilarFile } from "@/types";

interface SemanticSearchPanelProps {
  searchResults: SimilarFile[];
  searching: boolean;
  embeddingAll: boolean;
  hasFiles: boolean;
  onSearch: (query: string) => void;
  onEmbedAll: () => void;
}

export function SemanticSearchPanel({
  searchResults, searching, embeddingAll,
  hasFiles, onSearch, onEmbedAll,
}: SemanticSearchPanelProps) {
  const [localQuery, setLocalQuery] = useState("");

  // 空状态：未扫描文件
  if (!hasFiles) {
    return (
      <div className="flex flex-1 items-center justify-center">
        <div className="flex flex-col items-center gap-3 text-muted-foreground">
          <Radar className="h-10 w-10 opacity-30" />
          <p className="text-sm">请先在概览页完成文件扫描</p>
        </div>
      </div>
    );
  }

  return (
    <div className="flex flex-col gap-4">
      {/* 操作栏：Embedding 生成 + 搜索输入 */}
      <div className="flex items-center gap-2">
        <Button
          variant="outline"
          size="sm"
          onClick={onEmbedAll}
          disabled={embeddingAll}
          className="gap-2 shrink-0"
        >
          {embeddingAll ? <Loader2 className="h-3.5 w-3.5 animate-spin" /> : <Database className="h-3.5 w-3.5" />}
          {embeddingAll ? "生成中..." : "生成 Embedding"}
        </Button>

        <div className="relative flex-1">
          <Radar className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
          <input
            type="text"
            value={localQuery}
            onChange={(e) => setLocalQuery(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === "Enter" && localQuery.trim()) onSearch(localQuery);
            }}
            placeholder="输入自然语言描述搜索相关文件..."
            className="w-full rounded-lg border border-border bg-background pl-10 pr-3 py-2 text-sm text-foreground outline-none focus:ring-1 focus:ring-ring"
          />
        </div>
        <Button
          onClick={() => onSearch(localQuery)}
          disabled={searching || !localQuery.trim()}
          variant="outline"
          className="gap-2 shrink-0"
        >
          {searching ? <Loader2 className="h-4 w-4 animate-spin" /> : <Search className="h-4 w-4" />}
          搜索
        </Button>
      </div>

      {/* 搜索结果 */}
      {searchResults.length > 0 && (
        <div className="flex flex-col gap-1">
          {searchResults.map((item) => (
            <div
              key={item.relative_path}
              className="flex items-start gap-3 rounded-lg border border-border/50 px-3 py-2 text-sm hover:bg-muted/30 transition-colors"
            >
              <FileText className="mt-0.5 h-4 w-4 shrink-0 text-muted-foreground" />
              <div className="flex-1 min-w-0">
                <div className="font-medium text-foreground truncate" title={item.relative_path}>
                  {item.relative_path}
                </div>
                {item.summary && (
                  <div className="mt-0.5 text-xs text-muted-foreground line-clamp-2">{item.summary}</div>
                )}
              </div>
              <span className="shrink-0 rounded-full bg-blue-100 px-2 py-0.5 text-xs text-blue-700">
                {(item.score * 100).toFixed(1)}%
              </span>
            </div>
          ))}
        </div>
      )}

      {/* 空状态：有文件但无搜索结果 */}
      {searchResults.length === 0 && !searching && (
        <div className="flex flex-1 items-center justify-center py-12">
          <div className="flex flex-col items-center gap-3 text-muted-foreground">
            <Radar className="h-10 w-10 opacity-30" />
            <p className="text-sm">先生成 Embedding，然后输入关键词搜索相关文件</p>
          </div>
        </div>
      )}
    </div>
  );
}
