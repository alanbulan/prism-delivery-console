/**
 * CoreFilesList - 核心文件列表组件（只读）
 *
 * 职责：
 * - 从 Zustand store 读取 coreFiles 列表并展示
 * - 以锁定图标标识文件不可选中
 * - 区分文件（如 main.py）和目录（如 config/）的显示图标
 * - 无项目加载时显示空状态提示
 *
 * 需求: 3.1 - 在界面左侧区域展示不可选的 Core_Files 列表
 */

import { File, Folder, Lock } from "lucide-react";
import { useAppStore } from "@/store";

/**
 * 判断条目是否为目录
 * 约定：以 "/" 结尾的条目视为目录
 */
function isDirectory(entry: string): boolean {
  return entry.endsWith("/");
}

export function CoreFilesList() {
  const coreFiles = useAppStore((s) => s.coreFiles);
  const projectPath = useAppStore((s) => s.projectPath);

  // 未加载项目时显示空状态提示
  if (!projectPath || coreFiles.length === 0) {
    return (
      <div className="flex flex-1 items-center justify-center">
        <p className="text-xs text-muted-foreground">
          打开项目后显示核心架构文件
        </p>
      </div>
    );
  }

  return (
    <ul className="flex flex-col gap-1">
      {coreFiles.map((entry) => {
        const dir = isDirectory(entry);
        // 目录名去掉末尾斜杠用于显示
        const displayName = dir ? entry.slice(0, -1) : entry;

        return (
          <li
            key={entry}
            className="flex items-center gap-2 rounded-md px-2 py-1.5 text-sm text-muted-foreground"
          >
            {/* 文件/目录类型图标 */}
            {dir ? (
              <Folder className="h-4 w-4 shrink-0 text-blue-400/80" />
            ) : (
              <File className="h-4 w-4 shrink-0 text-slate-400/80" />
            )}

            {/* 条目名称 */}
            <span className="truncate">{displayName}</span>

            {/* 锁定图标，表示不可选中 */}
            <Lock className="ml-auto h-3 w-3 shrink-0 text-muted-foreground/50" />
          </li>
        );
      })}
    </ul>
  );
}
