/**
 * ProjectSelector - 项目选择器组件
 *
 * 职责：
 * - 提供"打开项目"按钮，调用 Tauri open_project command
 * - 显示当前已加载的项目路径
 * - 项目打开成功后自动调用 scan_modules 扫描业务模块
 * - 用户取消对话框时静默处理（需求 1.5）
 * - 其他错误通过 Toast 提示用户（需求 1.4）
 *
 * 需求: 1.1, 1.3, 1.4, 1.5
 */

import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import { useAppStore } from "@/store";
import type { ProjectInfo, ModuleInfo } from "@/types";

export function ProjectSelector() {
  const projectPath = useAppStore((s) => s.projectPath);
  const setProject = useAppStore((s) => s.setProject);
  const setModules = useAppStore((s) => s.setModules);

  /** 是否正在加载中（打开项目 + 扫描模块） */
  const [loading, setLoading] = useState(false);

  /**
   * 处理"打开项目"按钮点击
   * 1. 调用 open_project 打开原生文件夹选择对话框并验证项目结构
   * 2. 成功后将项目信息写入 store
   * 3. 自动调用 scan_modules 扫描业务模块
   */
  const handleOpenProject = async () => {
    setLoading(true);
    try {
      // 调用 Rust 后端打开项目（弹出文件夹选择对话框 + 验证）
      const info = await invoke<ProjectInfo>("open_project");

      // 设置项目信息到 store（同时会重置之前的模块选择状态）
      setProject(info.path, info.core_files);

      // 项目验证通过后，自动扫描业务模块（需求 1.3）
      try {
        const modules = await invoke<ModuleInfo[]>("scan_modules", {
          projectPath: info.path,
        });
        setModules(modules);
      } catch (scanErr) {
        // 模块扫描失败，通过 Toast 提示
        toast.error(String(scanErr));
      }
    } catch (err) {
      const errMsg = String(err);
      // 用户取消对话框时静默处理（需求 1.5）
      if (errMsg === "cancelled") {
        return;
      }
      // 其他错误通过 Toast 显示（需求 1.4）
      toast.error(errMsg);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="flex items-center gap-3">
      {/* 应用标题 */}
      <span className="font-medium text-foreground">FastBuilder Slogan</span>
      <span className="text-border">|</span>

      {/* 打开项目按钮（需求 1.1） */}
      <Button
        variant="outline"
        size="sm"
        onClick={handleOpenProject}
        disabled={loading}
      >
        {loading ? "加载中..." : "打开项目"}
      </Button>

      {/* 当前项目路径显示 */}
      {projectPath ? (
        <span className="truncate text-sm text-muted-foreground" title={projectPath}>
          {projectPath}
        </span>
      ) : (
        <span className="text-sm text-muted-foreground">
          请选择 FastAPI 主仓库文件夹
        </span>
      )}
    </div>
  );
}
