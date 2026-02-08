/**
 * 项目管理页面 - 本地类型定义
 *
 * 防止循环依赖，将页面内部使用的 Props 接口集中定义
 */

import type { Category, Project } from "@/types";

/** 技术栈选项常量 */
export const TECH_STACK_OPTIONS = [
  { value: "fastapi", label: "FastAPI" },
  { value: "vue3", label: "Vue 3" },
] as const;

/** 分类表单弹窗 Props */
export interface CategoryFormProps {
  /** 编辑时传入已有分类，新建时为 null */
  category: Category | null;
  onClose: () => void;
  onSaved: () => void;
}

/** 项目表单弹窗 Props */
export interface ProjectFormProps {
  /** 编辑时传入已有项目，新建时为 null */
  project: Project | null;
  /** 当前可用的分类列表 */
  categories: Category[];
  /** 新建时默认选中的分类 ID */
  defaultCategoryId: number | null;
  onClose: () => void;
  onSaved: () => void;
}

/** 分类侧边栏 Props */
export interface CategorySidebarProps {
  categories: Category[];
  selectedCategoryId: number | null;
  onSelect: (id: number | null) => void;
  onAdd: () => void;
  onEdit: (category: Category) => void;
  onDelete: (category: Category) => void;
}

/** 项目卡片 Props */
export interface ProjectCardProps {
  project: Project;
  onEdit: (project: Project) => void;
  onDelete: (project: Project) => void;
  onClick: (project: Project) => void;
}

/** 删除目标联合类型 */
export type DeleteTarget =
  | { type: "category"; item: Category }
  | { type: "project"; item: Project }
  | null;
