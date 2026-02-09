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
  /** 包含的模块数量（含自动补充的依赖模块） */
  module_count: number;
  /** 实际打包的完整模块列表（用户选中 + 依赖分析自动补充） */
  expanded_modules: string[];
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
  /** 模块扫描目录（相对于 repo_path，如 "modules"、"src/views"） */
  modules_dir: string;
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
  /** 构建版本号（如 v1.0.0） */
  version: string;
  /** 变更日志（与上次构建的模块差异，可为 null） */
  changelog: string | null;
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

/** LLM API 配置 */
export interface LlmConfig {
  /** OpenAI 兼容 API 基础地址（如 http://localhost:11434/v1） */
  base_url: string;
  /** API Key */
  api_key: string;
  /** 模型名称 */
  model_name: string;
  /** Embedding 模型名称 */
  embedding_model: string;
}

/** LLM 模型信息（从 /v1/models 获取） */
export interface LlmModel {
  /** 模型 ID */
  id: string;
}

/** 页面导航标识 */
export type PageId = 'projects' | 'build' | 'analysis' | 'settings' | 'about';

/** 文件索引条目（由 scan_project_file_index 返回） */
export interface FileIndexEntry {
  /** 相对路径 */
  relative_path: string;
  /** SHA256 哈希 */
  file_hash: string;
  /** 是否有变更（与数据库中的哈希不同） */
  changed: boolean;
  /** LLM 生成的文件摘要（可为空） */
  summary: string | null;
}

/** 依赖边 */
export interface DepEdge {
  /** 源文件相对路径 */
  source: string;
  /** 目标文件相对路径 */
  target: string;
}

/** 依赖图数据（由 analyze_dependencies 返回） */
export interface DependencyGraph {
  /** 所有文件节点（相对路径） */
  nodes: string[];
  /** 依赖边列表 */
  edges: DepEdge[];
}

/** 语义搜索结果条目（由 search_similar_files 返回） */
export interface SimilarFile {
  /** 文件相对路径 */
  relative_path: string;
  /** 文件摘要 */
  summary: string | null;
  /** 余弦相似度分数 */
  score: number;
}

/** 批量 Embedding 结果（由 embed_all_files 返回） */
export interface EmbedBatchResult {
  /** 总文件数 */
  total: number;
  /** 成功数 */
  success: number;
  /** 失败数 */
  failed: number;
}

/** 语言统计条目（由 get_project_overview 返回） */
export interface LanguageStat {
  /** 语言名称 */
  language: string;
  /** 文件数量 */
  file_count: number;
  /** 总行数 */
  line_count: number;
}

/** 项目概览数据（由 get_project_overview 返回） */
export interface ProjectOverview {
  /** 总文件数 */
  total_files: number;
  /** 总代码行数 */
  total_lines: number;
  /** 总目录数 */
  total_dirs: number;
  /** 检测到的技术栈标签 */
  tech_stack: string[];
  /** 按语言分类的文件统计 */
  languages: LanguageStat[];
  /** 入口文件列表 */
  entry_files: string[];
}

/** 签名索引结果（由 index_project_signatures 返回） */
export interface IndexSignaturesResult {
  /** 总文件数 */
  total: number;
  /** 成功提取签名的文件数 */
  indexed: number;
}
