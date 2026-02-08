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

import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { toast } from "sonner";
import { FolderOpen, Database, Loader2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import type { AppSettings } from "@/types";

export function SettingsPage() {
  // ---- 本地状态 ----
  /** 默认构建输出目录 */
  const [outputDir, setOutputDir] = useState<string | null>(null);
  /** 数据库文件路径 */
  const [dbPath, setDbPath] = useState("");
  /** 是否正在加载设置 */
  const [loading, setLoading] = useState(true);

  // ---- 加载设置 ----

  /** 页面挂载时加载当前设置 */
  useEffect(() => {
    async function loadSettings() {
      try {
        const settings = await invoke<AppSettings>("get_app_settings");
        setOutputDir(settings.default_output_dir);
        setDbPath(settings.db_path);
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
      </main>
    </div>
  );
}
