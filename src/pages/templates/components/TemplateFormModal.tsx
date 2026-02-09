/**
 * TemplateFormModal - 模板新建/编辑弹窗
 *
 * 提供 6 个字段的表单输入：name, modules_dir, extra_excludes, entry_file, import_pattern, router_pattern
 */

import { useState } from "react";
import { X } from "lucide-react";
import { Modal } from "@/components/ui/modal";
import type { TechStackTemplate } from "@/types";

interface Props {
  /** 编辑时传入已有模板，新建时为 null */
  template: TechStackTemplate | null;
  onClose: () => void;
  onSave: (data: Omit<TechStackTemplate, "id" | "is_builtin" | "created_at">) => Promise<void>;
}

export function TemplateFormModal({ template, onClose, onSave }: Props) {
  const [name, setName] = useState(template?.name ?? "");
  const [modulesDir, setModulesDir] = useState(template?.modules_dir ?? "");
  const [extraExcludes, setExtraExcludes] = useState(template?.extra_excludes ?? "[]");
  const [entryFile, setEntryFile] = useState(template?.entry_file ?? "");
  const [importPattern, setImportPattern] = useState(template?.import_pattern ?? "");
  const [routerPattern, setRouterPattern] = useState(template?.router_pattern ?? "");
  const [saving, setSaving] = useState(false);
  const isEdit = template !== null;

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!name.trim()) return;
    if (!modulesDir.trim()) return;
    setSaving(true);
    try {
      await onSave({
        name: name.trim(),
        modules_dir: modulesDir.trim(),
        extra_excludes: extraExcludes.trim() || "[]",
        entry_file: entryFile.trim(),
        import_pattern: importPattern.trim(),
        router_pattern: routerPattern.trim(),
      });
      onClose();
    } catch (err) {
      // 错误已在 composable 中 toast
    } finally {
      setSaving(false);
    }
  };

  return (
    <Modal onClose={onClose}>
      <div className="mb-4 flex items-center justify-between">
        <h3 className="text-lg font-semibold text-foreground">
          {isEdit ? "编辑模板" : "新建模板"}
        </h3>
        <button
          type="button"
          onClick={onClose}
          className="rounded-md p-1 text-muted-foreground hover:bg-accent"
        >
          <X className="h-4 w-4" />
        </button>
      </div>
      <form onSubmit={handleSubmit} className="flex flex-col gap-3">
        {/* 模板名称 */}
        <div>
          <label className="mb-1 block text-sm font-medium text-foreground">
            模板名称
          </label>
          <input
            type="text"
            value={name}
            onChange={(e) => setName(e.target.value)}
            placeholder="如 django, springboot"
            className="w-full rounded-lg border border-border bg-background px-3 py-2 text-sm text-foreground outline-none focus:ring-2 focus:ring-ring"
            autoFocus
          />
        </div>
        {/* 模块目录 */}
        <div>
          <label className="mb-1 block text-sm font-medium text-foreground">
            模块目录
          </label>
          <input
            type="text"
            value={modulesDir}
            onChange={(e) => setModulesDir(e.target.value)}
            placeholder="如 modules, src/views, apps"
            className="w-full rounded-lg border border-border bg-background px-3 py-2 text-sm text-foreground outline-none focus:ring-2 focus:ring-ring"
          />
          <p className="mt-1 text-xs text-muted-foreground">
            相对于仓库根目录的模块扫描路径
          </p>
        </div>
        {/* 额外排除目录 */}
        <div>
          <label className="mb-1 block text-sm font-medium text-foreground">
            额外排除目录
          </label>
          <input
            type="text"
            value={extraExcludes}
            onChange={(e) => setExtraExcludes(e.target.value)}
            placeholder='["dist", ".next"]'
            className="w-full rounded-lg border border-border bg-background px-3 py-2 text-sm text-foreground outline-none focus:ring-2 focus:ring-ring"
          />
          <p className="mt-1 text-xs text-muted-foreground">
            JSON 数组格式，构建时额外排除的目录
          </p>
        </div>
        {/* 入口文件 */}
        <div>
          <label className="mb-1 block text-sm font-medium text-foreground">
            入口文件
          </label>
          <input
            type="text"
            value={entryFile}
            onChange={(e) => setEntryFile(e.target.value)}
            placeholder="如 main.py, src/router/index.ts"
            className="w-full rounded-lg border border-border bg-background px-3 py-2 text-sm text-foreground outline-none focus:ring-2 focus:ring-ring"
          />
          <p className="mt-1 text-xs text-muted-foreground">
            留空则跳过导入重写
          </p>
        </div>
        {/* 导入匹配正则 */}
        <div>
          <label className="mb-1 block text-sm font-medium text-foreground">
            导入匹配正则
          </label>
          <input
            type="text"
            value={importPattern}
            onChange={(e) => setImportPattern(e.target.value)}
            placeholder={'如 from\\s+{modules_dir}\\.(\\w+)'}
            className="w-full rounded-lg border border-border bg-background px-3 py-2 text-sm text-foreground outline-none focus:ring-2 focus:ring-ring"
          />
          <p className="mt-1 text-xs text-muted-foreground">
            {'第一个捕获组应为模块名，{modules_dir} 会被替换为实际目录'}
          </p>
        </div>
        {/* 路由注册正则 */}
        <div>
          <label className="mb-1 block text-sm font-medium text-foreground">
            路由注册正则
          </label>
          <input
            type="text"
            value={routerPattern}
            onChange={(e) => setRouterPattern(e.target.value)}
            placeholder={'如 include_router\\('}
            className="w-full rounded-lg border border-border bg-background px-3 py-2 text-sm text-foreground outline-none focus:ring-2 focus:ring-ring"
          />
        </div>
        {/* 操作按钮 */}
        <div className="flex justify-end gap-2 pt-2">
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
