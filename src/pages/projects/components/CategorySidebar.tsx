/**
 * CategorySidebar - 左侧分类列表组件
 *
 * 职责：
 * - 展示分类列表，支持选中高亮
 * - 提供"全部项目"选项
 * - 每个分类项悬停显示编辑/删除按钮
 */

import { Plus, Pencil, Trash2, Layers } from "lucide-react";
import type { CategorySidebarProps } from "../types";

export function CategorySidebar({
  categories,
  selectedCategoryId,
  onSelect,
  onAdd,
  onEdit,
  onDelete,
}: CategorySidebarProps) {
  return (
    <aside className="glass-panel flex w-56 shrink-0 flex-col p-3">
      {/* 标题 + 新建按钮 */}
      <div className="mb-3 flex items-center justify-between px-1">
        <h3 className="text-sm font-semibold text-foreground">分类</h3>
        <button
          type="button"
          onClick={onAdd}
          className="rounded-md p-1 text-muted-foreground hover:bg-accent hover:text-foreground"
          title="新建分类"
        >
          <Plus className="h-4 w-4" />
        </button>
      </div>

      {/* "全部" 选项 */}
      <button
        type="button"
        onClick={() => onSelect(null)}
        className={`mb-1 flex items-center gap-2 rounded-lg px-3 py-2 text-sm transition-colors ${
          selectedCategoryId === null
            ? "bg-primary text-primary-foreground font-medium"
            : "text-muted-foreground hover:bg-accent hover:text-foreground"
        }`}
      >
        <Layers className="h-3.5 w-3.5 shrink-0" />
        <span>全部项目</span>
      </button>

      {/* 分类列表 */}
      <div className="flex flex-1 flex-col gap-0.5 overflow-auto">
        {categories.map((cat) => {
          const isActive = selectedCategoryId === cat.id;
          return (
            <div
              key={cat.id}
              className={`group flex items-center rounded-lg transition-colors ${
                isActive
                  ? "bg-primary text-primary-foreground"
                  : "text-muted-foreground hover:bg-accent hover:text-foreground"
              }`}
            >
              {/* 分类名称按钮 */}
              <button
                type="button"
                onClick={() => onSelect(cat.id)}
                className="flex-1 truncate px-3 py-2 text-left text-sm"
                title={cat.description ?? cat.name}
              >
                {cat.name}
              </button>
              {/* 编辑/删除操作（悬停显示） */}
              <div className="flex shrink-0 gap-0.5 pr-1 opacity-0 transition-opacity group-hover:opacity-100">
                <button
                  type="button"
                  onClick={(e) => {
                    e.stopPropagation();
                    onEdit(cat);
                  }}
                  className={`rounded p-1 ${
                    isActive
                      ? "hover:bg-primary-foreground/20"
                      : "hover:bg-accent"
                  }`}
                  title="编辑分类"
                >
                  <Pencil className="h-3 w-3" />
                </button>
                <button
                  type="button"
                  onClick={(e) => {
                    e.stopPropagation();
                    onDelete(cat);
                  }}
                  className={`rounded p-1 ${
                    isActive
                      ? "hover:bg-primary-foreground/20"
                      : "hover:bg-accent"
                  }`}
                  title="删除分类"
                >
                  <Trash2 className="h-3 w-3" />
                </button>
              </div>
            </div>
          );
        })}
      </div>
    </aside>
  );
}
