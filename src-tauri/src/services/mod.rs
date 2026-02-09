// ============================================================================
// 业务层：纯 Rust 核心逻辑
// ✅ 特点：尽量不依赖 `tauri::*`，保持纯净，方便写 #[test]
// ⛔ 禁止：直接返回前端专用的错误格式
// ============================================================================

pub mod analyzer;
pub mod build_strategy;
pub mod llm_client;
pub mod module_rewriter;
pub mod packer;
pub mod scan_strategy;
pub mod scanner;

// ============================================================================
// 常量定义
// ============================================================================

/// 核心文件白名单：构建交付包时必须包含的文件和目录
/// 这些是 FastAPI 项目的核心架构文件，交付时不可缺少
pub const CORE_FILES: &[&str] = &[
    "main.py",
    "requirements.txt",
    ".env.example",
    "config/",
    "core/",
    "utils/",
];

/// 忽略条目列表：扫描 modules/ 目录时需要跳过的目录/文件名
pub const IGNORED_ENTRIES: &[&str] = &["__pycache__", ".git", ".DS_Store"];
/// 构建时默认排除的目录列表
/// 这些目录不应出现在交付包中（版本控制、依赖缓存、构建产物、敏感文件等）
pub const DEFAULT_EXCLUDES: &[&str] = &[
    ".git",
    "node_modules",
    "__pycache__",
    ".venv",
    "venv",
    "target",
    "dist",
    "build",
    ".next",
    ".nuxt",
    ".idea",
    ".vscode",
    ".DS_Store",
    ".pytest_cache",
    ".mypy_cache",
    ".tox",
    "eggs",
    "*.egg-info",
    // 敏感文件：防止泄露数据库密码、API Key 等
    ".env",
    ".env.local",
    ".env.production",
    ".env.development",
];
