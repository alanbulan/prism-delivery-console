/**
 * BuildHistory - 构建历史记录列表组件
 *
 * 职责：
 * - 展示构建历史记录
 * - 每条记录显示时间、客户名、模块数、输出路径
 * - 提供"打开文件夹"和"删除"按钮
 * - 提供"清空历史"和"按时间清洗"操作
 */

import { useState } from "react";
import { History, FolderOpen, Trash2, AlertTriangle } from "lucide-react";
import type { BuildRecord } from "@/types";

interface BuildHistoryProps {
  records: BuildRecord[];
  getClientName: (clientId: number) => string;
  getModuleCount: (modulesJson: string) => number;
  onOpenFolder: (outputPath: string) => void;
  onDeleteRecord: (recordId: number, deleteFiles: boolean) => void;
  onClearAll: (deleteFiles: boolean) => void;
  onPurge: (days: number, deleteFiles: boolean) => void;
}

export function BuildHistory({
  records,
  getClientName,
  getModuleCount,
  onOpenFolder,
  onDeleteRecord,
  onClearAll,
  onPurge,
}: BuildHistoryProps) {
  const [showPurgeInput, setShowPurgeInput] = useState(false);
  const [purgeDays, setPurgeDays] = useState("30");
  const [confirmClear, setConfirmClear] = useState(false);
  const [deleteFiles, setDeleteFiles] = useState(false);

  /** 处理清洗提交 */
  const handlePurgeSubmit = () => {
    const days = parseInt(purgeDays, 10);
    if (isNaN(days) || days <= 0) return;
    onPurge(days, deleteFiles);
    setShowPurgeInput(false);
    setDeleteFiles(false);
  };

  return (
    <section className="glass flex flex-col gap-3 p-4">
      {/* 标题栏 + 操作按钮 */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <History className="h-4 w-4 text-muted-foreground" />
          <h3 className="text-sm font-semibold text-foreground">构建历史</h3>
          {records.length > 0 && (
            <span className="text-xs text-muted-foreground">
              ({records.length} 条)
            </span>
          )}
        </div>

        {/* 操作按钮组 */}
        {records.length > 0 && (
          <div className="flex items-center gap-1.5">
            {/* 全局"删除文件"开关 */}
            <label className="flex items-center gap-1 rounded px-2 py-1 text-xs text-muted-foreground hover:bg-accent">
              <input
                type="checkbox"
                checked={deleteFiles}
                onChange={(e) => setDeleteFiles(e.target.checked)}
                className="h-3 w-3 rounded border-border"
              />
              含文件
            </label>
            <span className="text-border">|</span>
            {/* 按时间清洗 */}
            <button
              type="button"
              onClick={() => setShowPurgeInput(!showPurgeInput)}
              className="rounded px-2 py-1 text-xs text-muted-foreground hover:bg-accent hover:text-foreground"
            >
              按时间清洗
            </button>
            {/* 清空全部 */}
            {!confirmClear ? (
              <button
                type="button"
                onClick={() => setConfirmClear(true)}
                className="rounded px-2 py-1 text-xs text-destructive hover:bg-destructive/10"
              >
                清空全部
              </button>
            ) : (
              <div className="flex items-center gap-1">
                <AlertTriangle className="h-3 w-3 text-destructive" />
                <span className="text-xs text-destructive">确认清空？</span>
                <button
                  type="button"
                  onClick={() => { onClearAll(deleteFiles); setConfirmClear(false); }}
                  className="rounded bg-destructive px-2 py-0.5 text-xs text-destructive-foreground hover:opacity-90"
                >
                  确认
                </button>
                <button
                  type="button"
                  onClick={() => setConfirmClear(false)}
                  className="rounded px-2 py-0.5 text-xs text-muted-foreground hover:bg-accent"
                >
                  取消
                </button>
              </div>
            )}
          </div>
        )}
      </div>

      {/* 按时间清洗输入框 */}
      {showPurgeInput && (
        <div className="flex items-center gap-2 rounded-lg border border-border bg-background/50 px-3 py-2">
          <span className="text-xs text-muted-foreground">删除</span>
          <input
            type="number"
            min="1"
            value={purgeDays}
            onChange={(e) => setPurgeDays(e.target.value)}
            onKeyDown={(e) => e.key === "Enter" && handlePurgeSubmit()}
            className="w-16 rounded border border-border bg-background px-2 py-1 text-center text-xs text-foreground outline-none focus:ring-1 focus:ring-ring"
            autoFocus
          />
          <span className="text-xs text-muted-foreground">天前的记录</span>
          <button
            type="button"
            onClick={handlePurgeSubmit}
            className="rounded bg-primary px-2.5 py-1 text-xs font-medium text-primary-foreground hover:opacity-90"
          >
            执行
          </button>
          <button
            type="button"
            onClick={() => { setShowPurgeInput(false); setDeleteFiles(false); }}
            className="rounded px-2 py-1 text-xs text-muted-foreground hover:bg-accent"
          >
            取消
          </button>
        </div>
      )}

      {/* 记录列表 */}
      {records.length > 0 ? (
        <div className="flex flex-col gap-1.5">
          {records.map((record) => (
            <div
              key={record.id}
              className="glass-subtle flex flex-col gap-1 px-3 py-2 text-sm"
            >
              {/* 主行信息 */}
              <div className="flex items-center justify-between">
                <div className="flex flex-1 items-center gap-4 overflow-hidden">
                  <span className="shrink-0 rounded bg-primary/10 px-1.5 py-0.5 text-xs font-medium text-primary">
                    {record.version}
                  </span>
                  <span className="shrink-0 text-xs text-muted-foreground">
                    {record.created_at}
                  </span>
                  <span className="shrink-0 font-medium text-foreground">
                    {getClientName(record.client_id)}
                  </span>
                  <span className="shrink-0 text-xs text-muted-foreground">
                    {getModuleCount(record.selected_modules)} 个模块
                  </span>
                  <span
                    className="truncate text-xs text-muted-foreground"
                    title={record.output_path}
                  >
                    {record.output_path}
                  </span>
                </div>

                {/* 操作按钮 */}
                <div className="ml-2 flex shrink-0 items-center gap-1">
                  <button
                    type="button"
                    onClick={() => onOpenFolder(record.output_path)}
                    className="rounded-md p-1 text-muted-foreground hover:bg-accent hover:text-foreground"
                    title="打开文件夹"
                  >
                    <FolderOpen className="h-3.5 w-3.5" />
                  </button>
                  <button
                    type="button"
                    onClick={() => onDeleteRecord(record.id, deleteFiles)}
                    className="rounded-md p-1 text-muted-foreground hover:bg-destructive/10 hover:text-destructive"
                    title={deleteFiles ? "删除记录并删除文件" : "删除此记录"}
                  >
                    <Trash2 className="h-3.5 w-3.5" />
                  </button>
                </div>
              </div>

              {/* 变更日志（如果有） */}
              {record.changelog && (
                <p className="pl-1 text-xs text-muted-foreground/70">
                  📋 {record.changelog}
                </p>
              )}
            </div>
          ))}
        </div>
      ) : (
        <p className="py-4 text-center text-sm text-muted-foreground">
          暂无构建记录
        </p>
      )}
    </section>
  );
}
