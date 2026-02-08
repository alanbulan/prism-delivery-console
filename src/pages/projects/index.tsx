/**
 * 项目管理页面 (ProjectPage) - 容器组件
 *
 * 职责：
 * - 编排子组件和业务逻辑
 * - 管理弹窗的显示/隐藏
 * - 不包含具体的 UI 实现细节
 *
 * 需求: 1.1, 1.2, 1.3, 1.4, 2.1, 2.2, 2.3, 2.4, 2.5, 2.6
 */

import { Plus, FolderOpen } from "lucide-react";
import { useProjectData } from "./composables/useProjectData";
import { CategorySidebar } from "./components/CategorySidebar";
import { ProjectCard } from "./components/ProjectCard";
import { CategoryFormModal } from "./components/CategoryFormModal";
import { ProjectFormModal } from "./components/ProjectFormModal";
import { ConfirmDeleteModal } from "@/components/ui/confirm-dialog";
import type { Project } from "@/types";

export function ProjectPage() {
  const {
    categories,
    filteredProjects,
    selectedCategoryId,
    categoryFormOpen,
    editingCategory,
    projectFormOpen,
    editingProject,
    deleteTarget,
    setSelectedCategoryId,
    handleAddCategory,
    handleEditCategory,
    handleDeleteCategory,
    confirmDeleteCategory,
    setCategoryFormOpen,
    handleAddProject,
    handleEditProject,
    handleDeleteProject,
    confirmDeleteProject,
    setProjectFormOpen,
    handleProjectClick,
    setDeleteTarget,
    loadCategories,
    loadProjects,
  } = useProjectData();

  return (
    <div className="flex flex-1 flex-col overflow-hidden">
      {/* 页面标题栏 */}
      <header className="glass-subtle flex items-center justify-between px-5 py-3">
        <h2 className="text-base font-semibold text-foreground">项目管理</h2>
        <button
          type="button"
          onClick={handleAddCategory}
          className="flex items-center gap-1.5 rounded-lg bg-primary px-3 py-1.5 text-sm font-medium text-primary-foreground hover:opacity-90"
        >
          <Plus className="h-3.5 w-3.5" />
          <span>新建分类</span>
        </button>
      </header>

      {/* 主内容区：左侧分类 + 右侧项目 */}
      <main className="flex flex-1 gap-4 overflow-hidden p-4">
        {/* 左侧分类列表 */}
        <CategorySidebar
          categories={categories}
          selectedCategoryId={selectedCategoryId}
          onSelect={setSelectedCategoryId}
          onAdd={handleAddCategory}
          onEdit={handleEditCategory}
          onDelete={handleDeleteCategory}
        />

        {/* 右侧项目列表 */}
        <section className="flex flex-1 flex-col overflow-auto">
          {/* 项目区域标题 + 新建按钮 */}
          <div className="mb-3 flex items-center justify-between">
            <h3 className="text-sm font-semibold text-foreground">
              {selectedCategoryId === null
                ? "全部项目"
                : categories.find((c) => c.id === selectedCategoryId)?.name ??
                  "项目"}
              <span className="ml-2 text-xs font-normal text-muted-foreground">
                ({filteredProjects.length})
              </span>
            </h3>
            <button
              type="button"
              onClick={handleAddProject}
              disabled={categories.length === 0}
              className="flex items-center gap-1.5 rounded-lg border border-border px-3 py-1.5 text-sm text-foreground hover:bg-accent disabled:opacity-50 disabled:cursor-not-allowed"
              title={
                categories.length === 0 ? "请先创建分类" : "新建项目"
              }
            >
              <Plus className="h-3.5 w-3.5" />
              <span>新建项目</span>
            </button>
          </div>

          {/* 项目卡片网格 */}
          {filteredProjects.length > 0 ? (
            <div className="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-3">
              {filteredProjects.map((proj) => (
                <ProjectCard
                  key={proj.id}
                  project={proj}
                  onEdit={handleEditProject}
                  onDelete={handleDeleteProject}
                  onClick={handleProjectClick}
                />
              ))}
            </div>
          ) : (
            <div className="flex flex-1 flex-col items-center justify-center text-muted-foreground">
              <FolderOpen className="mb-2 h-10 w-10 opacity-30" />
              <p className="text-sm">
                {categories.length === 0
                  ? "请先创建分类，再添加项目"
                  : "暂无项目，点击上方按钮新建"}
              </p>
            </div>
          )}
        </section>
      </main>

      {/* ---- 弹窗层 ---- */}

      {/* 分类表单弹窗 */}
      {categoryFormOpen && (
        <CategoryFormModal
          category={editingCategory}
          onClose={() => setCategoryFormOpen(false)}
          onSaved={loadCategories}
        />
      )}

      {/* 项目表单弹窗 */}
      {projectFormOpen && (
        <ProjectFormModal
          project={editingProject}
          categories={categories}
          defaultCategoryId={selectedCategoryId}
          onClose={() => setProjectFormOpen(false)}
          onSaved={loadProjects}
        />
      )}

      {/* 删除确认弹窗 */}
      {deleteTarget?.type === "category" && (
        <ConfirmDeleteModal
          message={`确定要删除分类「${deleteTarget.item.name}」吗？如果该分类下仍有项目，删除将会失败。`}
          onConfirm={confirmDeleteCategory}
          onClose={() => setDeleteTarget(null)}
        />
      )}
      {deleteTarget?.type === "project" && (
        <ConfirmDeleteModal
          message={`确定要删除项目「${(deleteTarget.item as Project).name}」吗？关联的客户绑定和构建记录也将被删除。`}
          onConfirm={confirmDeleteProject}
          onClose={() => setDeleteTarget(null)}
        />
      )}
    </div>
  );
}
