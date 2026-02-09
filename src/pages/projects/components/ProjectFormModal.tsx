/**
 * ProjectFormModal - 项目新建/编辑弹窗
 *
 * 职责：
 * - 提供项目名称、分类、仓库路径、技术栈的表单输入
 * - 新建或编辑项目，调用 Rust 后端 db_create_project / db_update_project
 * - 保存前调用 scan_project_modules 校验项目结构（严格模式：校验失败阻止保存）
 */

import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { toast } from "sonner";
import { X, FolderOpen } from "lucide-react";
import { Modal } from "@/components/ui/modal";
import { TECH_STACK_OPTIONS } from "../types";
import type { ProjectFormProps } from "../types";
import type { TechStackTemplate } from "@/types";

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
  const [modulesDir, setModulesDir] = useState(project?.modules_dir ?? "");
  const [saving, setSaving] = useState(false);
  const isEdit = project !== null;

  // 从数据库加载技术栈模板列表，回退到硬编码选项
  const [stackOptions, setStackOptions] = useState<{ value: string; label: string }[]>(
    TECH_STACK_OPTIONS as unknown as { value: string; label: string }[]
  );
  useEffect(() => {
    invoke<TechStackTemplate[]>("db_list_templates")
      .then((list) => {
        if (list.length > 0) {
          setStackOptions(list.map((t) => ({ value: t.name, label: t.name })));
        }
      })
      .catch(() => {
        // 加载失败时使用硬编码回退
      });
  }, []);

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
    const trimmedPath = repoPath.trim();
    if (!trimmedName) {
      toast.error("项目名称不能为空");
      return;
    }
    if (!categoryId) {
      toast.error("请选择分类");
      return;
    }
    if (!trimmedPath) {
      toast.error("请选择仓库路径");
      return;
    }

    setSaving(true);
    try {
      // 保存前校验项目结构（传入自定义模块目录）
      await invoke("scan_project_modules", {
        projectPath: trimmedPath,
        techStack: techStack,
        modulesDir: modulesDir.trim(),
      });

      if (isEdit) {
        await invoke("db_update_project", {
          id: project.id,
          name: trimmedName,
          categoryId: categoryId,
          repoPath: trimmedPath,
          techStack: techStack,
          modulesDir: modulesDir.trim(),
        });
        toast.success("项目已更新");
      } else {
        await invoke("db_create_project", {
          name: trimmedName,
          categoryId: categoryId,
          repoPath: trimmedPath,
          techStack: techStack,
          modulesDir: modulesDir.trim(),
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
        {/* 仓库路径（新建和编辑均可选择） */}
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
            {stackOptions.map((opt) => (
              <option key={opt.value} value={opt.value}>
                {opt.label}
              </option>
            ))}
          </select>
        </div>
        {/* 模块目录（相对路径，留空使用默认值） */}
        <div>
          <label className="mb-1 block text-sm font-medium text-foreground">
            模块目录
          </label>
          <input
            type="text"
            value={modulesDir}
            onChange={(e) => setModulesDir(e.target.value)}
            placeholder={techStack === "vue3" ? "src/views" : "modules"}
            className="w-full rounded-lg border border-border bg-background px-3 py-2 text-sm text-foreground outline-none focus:ring-2 focus:ring-ring"
          />
          <p className="mt-1 text-xs text-muted-foreground">
            相对于仓库根目录的路径，留空使用默认值
          </p>
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
