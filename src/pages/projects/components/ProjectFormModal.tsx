/**
 * ProjectFormModal - 项目新建/编辑弹窗
 *
 * 职责：
 * - 提供项目名称、分类、仓库路径、技术栈的表单输入
 * - 新建或编辑项目，调用 Rust 后端 db_create_project / db_update_project
 */

import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { toast } from "sonner";
import { X, FolderOpen } from "lucide-react";
import { Modal } from "@/components/ui/modal";
import { TECH_STACK_OPTIONS } from "../types";
import type { ProjectFormProps } from "../types";

export function ProjectFormModal({
  project,
  categories,
  defaultCategoryId,
  onClose,
  onSaved,
}: ProjectFormProps) {
  const [name, setName] = useState(project?.name ?? "");
  const [categoryId, setCategoryId] = useState<number>(
    project?.category_id ?? defaultCategoryId ?? categories[0]?.id ?? 0
  );
  const [repoPath, setRepoPath] = useState(project?.repo_path ?? "");
  const [techStack, setTechStack] = useState(
    project?.tech_stack_type ?? "fastapi"
  );
  const [saving, setSaving] = useState(false);
  const isEdit = project !== null;

  /** 打开文件夹选择器选择仓库路径 */
  const handlePickFolder = async () => {
    try {
      const selected = await open({ directory: true });
      if (selected) {
        setRepoPath(selected);
      }
    } catch (err) {
      toast.error(String(err));
    }
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    const trimmedName = name.trim();
    if (!trimmedName) {
      toast.error("项目名称不能为空");
      return;
    }
    if (!categoryId) {
      toast.error("请选择分类");
      return;
    }
    if (!isEdit && !repoPath.trim()) {
      toast.error("请选择仓库路径");
      return;
    }
    setSaving(true);
    try {
      if (isEdit) {
        await invoke("db_update_project", {
          id: project.id,
          name: trimmedName,
          categoryId: categoryId,
          techStack: techStack,
        });
        toast.success("项目已更新");
      } else {
        await invoke("db_create_project", {
          name: trimmedName,
          categoryId: categoryId,
          repoPath: repoPath.trim(),
          techStack: techStack,
        });
        toast.success("项目已创建");
      }
      onSaved();
      onClose();
    } catch (err) {
      toast.error(String(err));
    } finally {
      setSaving(false);
    }
  };

  return (
    <Modal onClose={onClose}>
      <div className="mb-4 flex items-center justify-between">
        <h3 className="text-lg font-semibold text-foreground">
          {isEdit ? "编辑项目" : "新建项目"}
        </h3>
        <button
          type="button"
          onClick={onClose}
          className="rounded-md p-1 text-muted-foreground hover:bg-accent"
        >
          <X className="h-4 w-4" />
        </button>
      </div>
      <form onSubmit={handleSubmit} className="flex flex-col gap-4">
        {/* 项目名称 */}
        <div>
          <label className="mb-1 block text-sm font-medium text-foreground">
            项目名称
          </label>
          <input
            type="text"
            value={name}
            onChange={(e) => setName(e.target.value)}
            placeholder="输入项目名称"
            className="w-full rounded-lg border border-border bg-background px-3 py-2 text-sm text-foreground outline-none focus:ring-2 focus:ring-ring"
            autoFocus
          />
        </div>
        {/* 所属分类 */}
        <div>
          <label className="mb-1 block text-sm font-medium text-foreground">
            所属分类
          </label>
          <select
            value={categoryId}
            onChange={(e) => setCategoryId(Number(e.target.value))}
            className="w-full rounded-lg border border-border bg-background px-3 py-2 text-sm text-foreground outline-none focus:ring-2 focus:ring-ring"
          >
            {categories.map((c) => (
              <option key={c.id} value={c.id}>
                {c.name}
              </option>
            ))}
          </select>
        </div>
        {/* 仓库路径（仅新建时可选择） */}
        {!isEdit && (
          <div>
            <label className="mb-1 block text-sm font-medium text-foreground">
              仓库路径
            </label>
            <div className="flex gap-2">
              <input
                type="text"
                value={repoPath}
                readOnly
                placeholder="点击右侧按钮选择文件夹"
                className="flex-1 rounded-lg border border-border bg-background px-3 py-2 text-sm text-foreground outline-none"
              />
              <button
                type="button"
                onClick={handlePickFolder}
                className="flex shrink-0 items-center gap-1 rounded-lg border border-border px-3 py-2 text-sm text-foreground hover:bg-accent"
              >
                <FolderOpen className="h-4 w-4" />
                <span>选择</span>
              </button>
            </div>
          </div>
        )}
        {/* 技术栈类型 */}
        <div>
          <label className="mb-1 block text-sm font-medium text-foreground">
            技术栈
          </label>
          <select
            value={techStack}
            onChange={(e) => setTechStack(e.target.value)}
            className="w-full rounded-lg border border-border bg-background px-3 py-2 text-sm text-foreground outline-none focus:ring-2 focus:ring-ring"
          >
            {TECH_STACK_OPTIONS.map((opt) => (
              <option key={opt.value} value={opt.value}>
                {opt.label}
              </option>
            ))}
          </select>
        </div>
        {/* 操作按钮 */}
        <div className="flex justify-end gap-2">
          <button
            type="button"
            onClick={onClose}
            className="rounded-lg px-4 py-2 text-sm text-muted-foreground hover:bg-accent"
          >
            取消
          </button>
          <button
            type="submit"
            disabled={saving}
            className="rounded-lg bg-primary px-4 py-2 text-sm font-medium text-primary-foreground hover:opacity-90 disabled:opacity-50"
          >
            {saving ? "保存中..." : "保存"}
          </button>
        </div>
      </form>
    </Modal>
  );
}
