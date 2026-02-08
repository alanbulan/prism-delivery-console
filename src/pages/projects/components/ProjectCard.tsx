/**
 * ProjectCard - 单个项目卡片组件
 *
 * 职责：
 * - 展示项目名称、技术栈标签、仓库路径
 * - 悬停显示编辑/删除按钮
 * - 点击卡片触发导航到构建页
 */

import { Pencil, Trash2 } from "lucide-react";
import { TECH_STACK_OPTIONS } from "../types";
import type { ProjectCardProps } from "../types";

export function ProjectCard({ project, onEdit, onDelete, onClick }: ProjectCardProps) {
  /** 获取技术栈显示标签 */
  const techLabel =
    TECH_STACK_OPTIONS.find((o) => o.value === project.tech_stack_type)?.label ??
    project.tech_stack_type;

  return (
    <div
      className="glass group cursor-pointer p-4 transition-shadow hover:shadow-lg"
      onClick={() => onClick(project)}
      role="button"
      tabIndex={0}
      onKeyDown={(e) => {
        if (e.key === "Enter" || e.key === " ") onClick(project);
      }}
    >
      {/* 卡片头部：名称 + 操作按钮 */}
      <div className="mb-2 flex items-start justify-between">
        <h4 className="text-sm font-semibold text-foreground truncate pr-2">
          {project.name}
        </h4>
        <div className="flex shrink-0 gap-0.5 opacity-0 transition-opacity group-hover:opacity-100">
          <button
            type="button"
            onClick={(e) => {
              e.stopPropagation();
              onEdit(project);
            }}
            className="rounded p-1 text-muted-foreground hover:bg-accent hover:text-foreground"
            title="编辑项目"
          >
            <Pencil className="h-3 w-3" />
          </button>
          <button
            type="button"
            onClick={(e) => {
              e.stopPropagation();
              onDelete(project);
            }}
            className="rounded p-1 text-muted-foreground hover:bg-accent hover:text-destructive"
            title="删除项目"
          >
            <Trash2 className="h-3 w-3" />
          </button>
        </div>
      </div>

      {/* 技术栈标签 */}
      <span className="mb-2 inline-block rounded-md bg-accent px-2 py-0.5 text-xs text-accent-foreground">
        {techLabel}
      </span>

      {/* 仓库路径 */}
      <p
        className="truncate text-xs text-muted-foreground"
        title={project.repo_path}
      >
        {project.repo_path}
      </p>
    </div>
  );
}
