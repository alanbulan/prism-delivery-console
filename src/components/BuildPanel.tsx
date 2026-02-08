/**
 * BuildPanel - 底部构建面板组件
 *
 * 职责：
 * - 提供客户名称输入框（需求 4.1）
 * - 构建按钮：验证前置条件后调用 build_package command（需求 4.2, 4.3）
 * - 构建中禁用按钮 + 加载动画（需求 5.1, 5.2）
 * - 成功时显示 Toast 通知 + "打开文件夹"按钮（需求 4.9, 4.10）
 * - 失败时显示包含错误原因的 Toast（需求 5.4）
 *
 * 需求: 4.1, 4.2, 4.3, 4.9, 4.10, 5.1, 5.2, 5.4
 */

import { invoke } from "@tauri-apps/api/core";
import { toast } from "sonner";
import { Loader2, Package } from "lucide-react";
import { Button } from "@/components/ui/button";
import { useAppStore } from "@/store";
import type { BuildResult } from "@/types";

export function BuildPanel() {
  const clientName = useAppStore((s) => s.clientName);
  const setClientName = useAppStore((s) => s.setClientName);
  const isBuilding = useAppStore((s) => s.isBuilding);
  const setBuildingState = useAppStore((s) => s.setBuildingState);
  const setBuildResult = useAppStore((s) => s.setBuildResult);
  const selectedModules = useAppStore((s) => s.selectedModules);
  const projectPath = useAppStore((s) => s.projectPath);

  /** 构建按钮是否禁用：构建中 / 客户名为空 / 未选模块 / 未加载项目 */
  const isBuildDisabled =
    isBuilding ||
    clientName.trim() === "" ||
    selectedModules.size === 0 ||
    projectPath === null;

  /**
   * 处理构建按钮点击
   * 调用 Rust 后端 build_package command，处理成功/失败结果
   */
  const handleBuild = async () => {
    setBuildingState(true);
    setBuildResult(null);

    try {
      // 调用 Rust 后端构建交付包
      // Tauri 自动将 camelCase 参数转换为 snake_case
      const result = await invoke<BuildResult>("build_package", {
        projectPath,
        selectedModules: Array.from(selectedModules),
        clientName: clientName.trim(),
      });

      // 保存构建结果到 store
      setBuildResult(result);

      // 成功 Toast：显示构建信息 + "打开文件夹"操作按钮（需求 4.9, 4.10）
      toast.success(
        `构建完成：已打包 ${result.module_count} 个模块`,
        {
          action: {
            label: "打开文件夹",
            onClick: () => {
              // 调用 Rust 后端在系统文件管理器中打开 ZIP 所在目录
              invoke("open_folder", { path: result.zip_path }).catch((err) => {
                toast.error(`打开文件夹失败：${String(err)}`);
              });
            },
          },
        },
      );
    } catch (err) {
      // 失败 Toast：显示错误原因（需求 5.4）
      toast.error(`构建失败：${String(err)}`);
    } finally {
      // 无论成功或失败，重置构建状态
      setBuildingState(false);
    }
  };

  return (
    <div className="flex flex-1 items-center gap-3">
      {/* 客户名称输入框（需求 4.1） */}
      <input
        type="text"
        placeholder="输入客户名称"
        value={clientName}
        onChange={(e) => setClientName(e.target.value)}
        disabled={isBuilding}
        className="h-8 w-48 rounded-md border border-border bg-background/60 px-3 text-sm text-foreground placeholder:text-muted-foreground focus:border-ring focus:outline-none focus:ring-1 focus:ring-ring/50 disabled:opacity-50"
      />

      {/* 构建按钮（需求 4.2, 5.1, 5.2） */}
      <Button
        size="sm"
        disabled={isBuildDisabled}
        onClick={handleBuild}
      >
        {isBuilding ? (
          <>
            {/* 构建中加载动画（需求 5.1） */}
            <Loader2 className="h-3.5 w-3.5 animate-spin" />
            构建中...
          </>
        ) : (
          <>
            <Package className="h-3.5 w-3.5" />
            构建交付包
          </>
        )}
      </Button>

      {/* 已选模块计数提示 */}
      <span className="ml-auto text-xs text-muted-foreground">
        {selectedModules.size > 0
          ? `已选 ${selectedModules.size} 个模块`
          : "请先选择业务模块"}
      </span>
    </div>
  );
}
