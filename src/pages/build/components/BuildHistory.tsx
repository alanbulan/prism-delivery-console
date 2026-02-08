/**
 * BuildHistory - 构建历史记录列表组件
 *
 * 职责：
 * - 展示构建历史记录
 * - 每条记录显示时间、客户名、模块数、输出路径
 * - 提供"打开文件夹"按钮
 */

import { History, FolderOpen } from "lucide-react";
import type { BuildRecord } from "@/types";

interface BuildHistoryProps {
  records: BuildRecord[];
  getClientName: (clientId: number) => string;
  getModuleCount: (modulesJson: string) => number;
  onOpenFolder: (outputPath: string) => void;
}

export function BuildHistory({
  records,
  getClientName,
  getModuleCount,
  onOpenFolder,
}: BuildHistoryProps) {
  return (
    <section className="glass flex flex-col gap-3 p-4">
      <div className="flex items-center gap-2">
        <History className="h-4 w-4 text-muted-foreground" />
        <h3 className="text-sm font-semibold text-foreground">构建历史</h3>
      </div>

      {records.length > 0 ? (
        <div className="flex flex-col gap-1.5">
          {records.map((record) => (
            <div
              key={record.id}
              className="glass-subtle flex items-center justify-between px-3 py-2 text-sm"
            >
              {/* 左侧信息 */}
              <div className="flex flex-1 items-center gap-4 overflow-hidden">
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

              {/* 打开文件夹按钮 */}
              <button
                type="button"
                onClick={() => onOpenFolder(record.output_path)}
                className="ml-2 shrink-0 rounded-md p-1 text-muted-foreground hover:bg-accent hover:text-foreground"
                title="打开文件夹"
              >
                <FolderOpen className="h-3.5 w-3.5" />
              </button>
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
