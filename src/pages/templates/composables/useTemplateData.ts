/**
 * 模板管理 - 数据逻辑层
 *
 * 封装技术栈模板的 CRUD 操作和状态管理
 */

import { useState, useCallback, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { toast } from "sonner";
import type { TechStackTemplate } from "@/types";

export function useTemplateData() {
  const [templates, setTemplates] = useState<TechStackTemplate[]>([]);
  const [loading, setLoading] = useState(false);

  /** 加载所有模板 */
  const loadTemplates = useCallback(async () => {
    setLoading(true);
    try {
      const list = await invoke<TechStackTemplate[]>("db_list_templates");
      setTemplates(list);
    } catch (err) {
      toast.error(String(err));
    } finally {
      setLoading(false);
    }
  }, []);

  /** 创建模板 */
  const createTemplate = useCallback(
    async (data: Omit<TechStackTemplate, "id" | "is_builtin" | "created_at">) => {
      await invoke("db_create_template", {
        name: data.name,
        modulesDir: data.modules_dir,
        extraExcludes: data.extra_excludes,
        entryFile: data.entry_file,
        importPattern: data.import_pattern,
        routerPattern: data.router_pattern,
      });
      toast.success("模板已创建");
      await loadTemplates();
    },
    [loadTemplates]
  );

  /** 更新模板 */
  const updateTemplate = useCallback(
    async (id: number, data: Omit<TechStackTemplate, "id" | "is_builtin" | "created_at">) => {
      await invoke("db_update_template", {
        id,
        name: data.name,
        modulesDir: data.modules_dir,
        extraExcludes: data.extra_excludes,
        entryFile: data.entry_file,
        importPattern: data.import_pattern,
        routerPattern: data.router_pattern,
      });
      toast.success("模板已更新");
      await loadTemplates();
    },
    [loadTemplates]
  );

  /** 删除模板 */
  const deleteTemplate = useCallback(
    async (id: number) => {
      await invoke("db_delete_template", { id });
      toast.success("模板已删除");
      await loadTemplates();
    },
    [loadTemplates]
  );

  /** 导出模板为 JSON */
  const exportTemplate = useCallback(async (id: number) => {
    const json = await invoke<string>("export_template_json", { id });
    return json;
  }, []);

  /** 从 JSON 导入模板 */
  const importTemplate = useCallback(
    async (jsonStr: string) => {
      await invoke("import_template_json", { jsonStr });
      toast.success("模板已导入");
      await loadTemplates();
    },
    [loadTemplates]
  );

  // 初始加载
  useEffect(() => {
    loadTemplates();
  }, [loadTemplates]);

  return {
    templates,
    loading,
    loadTemplates,
    createTemplate,
    updateTemplate,
    deleteTemplate,
    exportTemplate,
    importTemplate,
  };
}
