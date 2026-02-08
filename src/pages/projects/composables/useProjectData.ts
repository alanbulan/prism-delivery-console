/**
 * useProjectData - 项目管理页面数据加载逻辑
 *
 * 职责：
 * - 加载分类列表和项目列表
 * - 提供分类/项目的 CRUD 操作
 * - 管理弹窗状态和删除确认
 */

import { useEffect, useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { toast } from "sonner";
import { useAppStore } from "@/store";
import type { Category, Project } from "@/types";
import type { DeleteTarget } from "../types";

export function useProjectData() {
  // ---- 全局 Store ----
  const categories = useAppStore((s) => s.categories);
  const projects = useAppStore((s) => s.projects);
  const setCategories = useAppStore((s) => s.setCategories);
  const setProjects = useAppStore((s) => s.setProjects);
  const setSelectedProjectId = useAppStore((s) => s.setSelectedProjectId);
  const setCurrentPage = useAppStore((s) => s.setCurrentPage);

  // ---- 本地状态 ----
  /** 当前选中的分类 ID，null 表示"全部" */
  const [selectedCategoryId, setSelectedCategoryId] = useState<number | null>(null);

  /** 分类表单弹窗状态 */
  const [categoryFormOpen, setCategoryFormOpen] = useState(false);
  const [editingCategory, setEditingCategory] = useState<Category | null>(null);

  /** 项目表单弹窗状态 */
  const [projectFormOpen, setProjectFormOpen] = useState(false);
  const [editingProject, setEditingProject] = useState<Project | null>(null);

  /** 删除确认弹窗状态 */
  const [deleteTarget, setDeleteTarget] = useState<DeleteTarget>(null);

  // ---- 数据加载 ----

  const loadCategories = useCallback(async () => {
    try {
      const list = await invoke<Category[]>("db_list_categories");
      setCategories(list);
    } catch (err) {
      toast.error(String(err));
    }
  }, [setCategories]);

  const loadProjects = useCallback(async () => {
    try {
      const list = await invoke<Project[]>("db_list_projects");
      setProjects(list);
    } catch (err) {
      toast.error(String(err));
    }
  }, [setProjects]);

  /** 页面挂载时加载数据 */
  useEffect(() => {
    loadCategories();
    loadProjects();
  }, [loadCategories, loadProjects]);

  // ---- 按选中分类过滤项目 ----
  const filteredProjects =
    selectedCategoryId === null
      ? projects
      : projects.filter((p) => p.category_id === selectedCategoryId);

  // ---- 分类操作 ----

  const handleAddCategory = () => {
    setEditingCategory(null);
    setCategoryFormOpen(true);
  };

  const handleEditCategory = (cat: Category) => {
    setEditingCategory(cat);
    setCategoryFormOpen(true);
  };

  const handleDeleteCategory = (cat: Category) => {
    setDeleteTarget({ type: "category", item: cat });
  };

  const confirmDeleteCategory = async () => {
    if (deleteTarget?.type !== "category") return;
    try {
      await invoke("db_delete_category", { id: deleteTarget.item.id });
      toast.success("分类已删除");
      if (selectedCategoryId === deleteTarget.item.id) {
        setSelectedCategoryId(null);
      }
      await loadCategories();
    } catch (err) {
      toast.error(String(err));
    } finally {
      setDeleteTarget(null);
    }
  };

  // ---- 项目操作 ----

  const handleAddProject = () => {
    setEditingProject(null);
    setProjectFormOpen(true);
  };

  const handleEditProject = (proj: Project) => {
    setEditingProject(proj);
    setProjectFormOpen(true);
  };

  const handleDeleteProject = (proj: Project) => {
    setDeleteTarget({ type: "project", item: proj });
  };

  const confirmDeleteProject = async () => {
    if (deleteTarget?.type !== "project") return;
    try {
      await invoke("db_delete_project", { id: deleteTarget.item.id });
      toast.success("项目已删除");
      await loadProjects();
    } catch (err) {
      toast.error(String(err));
    } finally {
      setDeleteTarget(null);
    }
  };

  /** 点击项目卡片：导航到构建页面 */
  const handleProjectClick = (proj: Project) => {
    setSelectedProjectId(proj.id);
    setCurrentPage("build");
  };

  return {
    // 数据
    categories,
    projects,
    filteredProjects,
    selectedCategoryId,

    // 弹窗状态
    categoryFormOpen,
    editingCategory,
    projectFormOpen,
    editingProject,
    deleteTarget,

    // 分类操作
    setSelectedCategoryId,
    handleAddCategory,
    handleEditCategory,
    handleDeleteCategory,
    confirmDeleteCategory,
    setCategoryFormOpen,

    // 项目操作
    handleAddProject,
    handleEditProject,
    handleDeleteProject,
    confirmDeleteProject,
    setProjectFormOpen,
    handleProjectClick,

    // 删除弹窗
    setDeleteTarget,

    // 数据刷新
    loadCategories,
    loadProjects,
  };
}
