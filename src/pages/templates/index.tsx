/**
 * 模板管理页面
 *
 * 提供技术栈模板的 CRUD 操作、JSON 导入/导出功能。
 * 内置模板（fastapi/vue3）只读，自定义模板可编辑/删除。
 */

import { useState } from "react";
import { Plus, Download, Upload, Pencil, Trash2, Lock } from "lucide-react";
import { toast } from "sonner";
import { useTemplateData } from "./composables/useTemplateData";
import { TemplateFormModal } from "./components/TemplateFormModal";
import { ConfirmDialog } from "@/components/ui/confirm-dialog";
import type { TechStackTemplate } from "@/types";

export default function TemplatesPage() {
  const {
    templates,
    loading,
    createTemplate,
    updateTemplate,
    deleteTemplate,
    exportTemplate,
    importTemplate,
  } = useTemplateData();

  // 弹窗状态
  const [formOpen, setFormOpen] = useState(false);
  const [editTarget, setEditTarget] = useState<TechStackTemplate | null>(null);
  const [deleteTarget, setDeleteTarget] = useState<TechStackTemplate | null>(null);

  /** 打开新建弹窗 */
  const handleAdd = () => {
    setEditTarget(null);
    setFormOpen(true);
  };

  /** 打开编辑弹窗 */
  const handleEdit = (t: TechStackTemplate) => {
    setEditTarget(t);
    setFormOpen(true);
  };

  /** 导出模板 JSON 到剪贴板 */
  const handleExport = async (t: TechStackTemplate) => {
    try {
      const json = await exportTemplate(t.id);
      await navigator.clipboard.writeText(json);
      toast.success("模板 JSON 已复制到剪贴板");
    } catch (err) {
      toast.error(String(err));
    }
  };

  /** 从剪贴板导入模板 JSON */
  const handleImport = async () => {
    try {
      const json = await navigator.clipboard.readText();
      if (!json.trim()) {
        toast.error("剪贴板为空");
        return;
      }
      await importTemplate(json);
    } catch (err) {
      toast.error(String(err));
    }
  };

  /** 保存表单（新建或编辑） */
  const handleSave = async (
    data: Omit<TechStackTemplate, "id" | "is_builtin" | "created_at">
  ) => {
    if (editTarget) {
      await updateTemplate(editTarget.id, data);
    } else {
      await createTemplate(data);
    }
  };

  /** 确认删除 */
  const handleConfirmDelete = async () => {
    if (!deleteTarget) return;
    try {
      await deleteTemplate(deleteTarget.id);
    } catch (err) {
      toast.error(String(err));
    }
    setDeleteTarget(null);
  };

  return (
    <div className="flex h-full flex-col gap-4 p-6">
      {/* 顶部操作栏 */}
      <div className="flex items-center justify-between">
        <h2 className="text-lg font-semibold text-foreground">技术栈模板</h2>
        <div className="flex gap-2">
          <button
            type="button"
            onClick={handleImport}
            className="flex items-center gap-1.5 rounded-lg border border-border px-3 py-1.5 text-sm text-foreground hover:bg-accent"
          >
            <Upload className="h-3.5 w-3.5" />
            从剪贴板导入
          </button>
          <button
            type="button"
            onClick={handleAdd}
            className="flex items-center gap-1.5 rounded-lg bg-primary px-3 py-1.5 text-sm font-medium text-primary-foreground hover:opacity-90"
          >
            <Plus className="h-3.5 w-3.5" />
            新建模板
          </button>
        </div>
      </div>

      {/* 模板列表 */}
      {loading ? (
        <div className="flex flex-1 items-center justify-center text-sm text-muted-foreground">
          加载中...
        </div>
      ) : templates.length === 0 ? (
        <div className="flex flex-1 items-center justify-center text-sm text-muted-foreground">
          暂无模板，点击右上角新建
        </div>
      ) : (
        <div className="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-3">
          {templates.map((t) => (
            <div
              key={t.id}
              className="flex flex-col gap-2 rounded-xl border border-border bg-card/60 p-4 backdrop-blur-sm"
            >
              {/* 模板名称 + 内置标记 */}
              <div className="flex items-center gap-2">
                <span className="text-sm font-semibold text-foreground">
                  {t.name}
                </span>
                {t.is_builtin && (
                  <span className="flex items-center gap-0.5 rounded-full bg-blue-100 px-2 py-0.5 text-xs text-blue-700">
                    <Lock className="h-3 w-3" />
                    内置
                  </span>
                )}
              </div>
              {/* 关键字段展示 */}
              <div className="flex flex-col gap-1 text-xs text-muted-foreground">
                <span>模块目录: {t.modules_dir}</span>
                {t.entry_file && <span>入口文件: {t.entry_file}</span>}
              </div>
              {/* 操作按钮 */}
              <div className="mt-auto flex items-center gap-1 pt-2">
                <button
                  type="button"
                  onClick={() => handleExport(t)}
                  className="flex items-center gap-1 rounded-md border border-border px-2 py-1 text-xs text-foreground hover:bg-accent"
                  title="导出 JSON"
                >
                  <Download className="h-3 w-3" />
                  导出
                </button>
                {!t.is_builtin && (
                  <>
                    <button
                      type="button"
                      onClick={() => handleEdit(t)}
                      className="flex items-center gap-1 rounded-md border border-border px-2 py-1 text-xs text-foreground hover:bg-accent"
                      title="编辑"
                    >
                      <Pencil className="h-3 w-3" />
                      编辑
                    </button>
                    <button
                      type="button"
                      onClick={() => setDeleteTarget(t)}
                      className="flex items-center gap-1 rounded-md border border-red-200 px-2 py-1 text-xs text-red-600 hover:bg-red-50"
                      title="删除"
                    >
                      <Trash2 className="h-3 w-3" />
                      删除
                    </button>
                  </>
                )}
              </div>
            </div>
          ))}
        </div>
      )}

      {/* 新建/编辑弹窗 */}
      {formOpen && (
        <TemplateFormModal
          template={editTarget}
          onClose={() => setFormOpen(false)}
          onSave={handleSave}
        />
      )}

      {/* 删除确认弹窗 */}
      {deleteTarget && (
        <ConfirmDialog
          title="删除模板"
          message={`确定要删除模板「${deleteTarget.name}」吗？此操作不可撤销。`}
          onConfirm={handleConfirmDelete}
          onCancel={() => setDeleteTarget(null)}
        />
      )}
    </div>
  );
}