/**
 * 前端类型定义
 * 与 Rust 后端 Tauri Commands 的返回值类型一一对应
 */

/** 项目信息，由 open_project command 返回 */
export interface ProjectInfo {
  /** 项目根目录路径 */
  path: string;
  /** 核心架构文件列表（白名单中实际存在的文件） */
  core_files: string[];
}

/** 业务模块信息，由 scan_modules command 返回 */
export interface ModuleInfo {
  /** 模块名称（modules/ 下的一级子目录名） */
  name: string;
  /** 模块完整路径 */
  path: string;
}

/** 构建结果，由 build_package command 返回 */
export interface BuildResult {
  /** 生成的 ZIP 交付包路径 */
  zip_path: string;
  /** 客户名称 */
  client_name: string;
  /** 包含的模块数量 */
  module_count: number;
}

// ============================================================
// V2 数据类型 - 多项目、多技术栈管理
// ============================================================

/** 项目分类，对应数据库 categories 表 */
export interface Category {
  /** 分类唯一标识 */
  id: number;
  /** 分类名称（唯一） */
  name: string;
  /** 分类描述（可选） */
  description: string | null;
  /** 创建时间 */
  created_at: string;
}

/** 项目记录，对应数据库 projects 表 */
export interface Project {
  /** 项目唯一标识 */
  id: number;
  /** 项目名称 */
  name: string;
  /** 所属分类 ID */
  category_id: number;
  /** 仓库路径 */
  repo_path: string;
  /** 技术栈类型（如 fastapi, vue3） */
  tech_stack_type: string;
  /** 创建时间 */
  created_at: string;
  /** 更新时间 */
  updated_at: string;
}

/** 交付客户，对应数据库 clients 表 */
export interface Client {
  /** 客户唯一标识 */
  id: number;
  /** 客户名称 */
  name: string;
  /** 创建时间 */
  created_at: string;
}

/** 构建历史记录，对应数据库 build_records 表 */
export interface BuildRecord {
  /** 记录唯一标识 */
  id: number;
  /** 关联项目 ID */
  project_id: number;
  /** 关联客户 ID */
  client_id: number;
  /** 选中的模块列表（JSON 字符串） */
  selected_modules: string;
  /** 构建输出路径 */
  output_path: string;
  /** 创建时间 */
  created_at: string;
}

/** 应用全局设置 */
export interface AppSettings {
  /** 默认构建输出目录（可选） */
  default_output_dir: string | null;
  /** 数据库文件路径 */
  db_path: string;
}

/** 页面导航标识 */
export type PageId = 'projects' | 'build' | 'quick-build' | 'settings' | 'about';
