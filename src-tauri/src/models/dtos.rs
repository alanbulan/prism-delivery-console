// ============================================================================
// 数据传输对象（DTO）定义
// 前后端通信的数据结构，仅包含字段定义和序列化派生
// ⛔ 禁止：包含复杂的业务逻辑方法
// ============================================================================

use serde::{Deserialize, Serialize};

/// 项目信息，由 `open_project` command 返回
/// 包含项目路径和实际存在的核心文件列表
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ProjectInfo {
    /// 项目根目录的绝对路径
    pub path: String,
    /// 核心文件白名单中实际存在的文件/目录列表
    pub core_files: Vec<String>,
}

/// 模块信息，由 `scan_modules` / `scan_project_modules` command 返回
/// 代表项目中的一个业务模块（如 modules/ 或 src/views/ 下的子目录）
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ModuleInfo {
    /// 模块名称（即子目录名）
    pub name: String,
    /// 模块的完整路径
    pub path: String,
}

/// 构建结果，由 `build_package` / `build_project_package` command 返回
/// 包含生成的 ZIP 交付包信息
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BuildResult {
    /// 生成的 ZIP 文件的完整路径
    pub zip_path: String,
    /// 客户名称
    pub client_name: String,
    /// 包含的业务模块数量（含自动补充的依赖模块）
    pub module_count: usize,
    /// 实际打包的完整模块列表（用户选中 + 依赖分析自动补充）
    /// 前端应使用此字段保存构建记录，而非原始 selectedModules
    pub expanded_modules: Vec<String>,
}
