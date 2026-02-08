// ============================================================================
// 项目相关 Commands
// 负责：项目打开、模块扫描（含多技术栈）
// ============================================================================

use crate::models::dtos::{ModuleInfo, ProjectInfo};
use crate::services::scan_strategy;
use crate::services::scanner;

/// 打开项目：弹出原生文件夹选择对话框，验证项目结构
///
/// 调用系统原生文件夹选择对话框，用户选择文件夹后验证其是否为有效的
/// FastAPI 项目（包含 main.py 和 modules/ 目录），并返回项目信息。
#[tauri::command]
pub async fn open_project(app: tauri::AppHandle) -> Result<ProjectInfo, String> {
    use tauri_plugin_dialog::DialogExt;

    // 调用原生文件夹选择对话框（阻塞等待用户选择）
    let folder = app.dialog().file().blocking_pick_folder();

    // 用户取消了对话框
    let folder_path = match folder {
        Some(f) => f,
        None => return Err("cancelled".to_string()),
    };

    // 解析文件夹路径
    let path = folder_path
        .as_path()
        .ok_or_else(|| "项目验证失败：无法解析所选文件夹路径".to_string())?;

    // 调用 services 层验证项目结构并扫描核心文件
    let core_files = scanner::validate_project(path)?;

    Ok(ProjectInfo {
        path: path.to_string_lossy().to_string(),
        core_files,
    })
}

/// 扫描模块：读取 modules/ 下的一级子目录，过滤忽略项
///
/// 接收项目路径，拼接 modules/ 子目录后调用 services 层执行扫描。
#[tauri::command]
pub async fn scan_modules(project_path: String) -> Result<Vec<ModuleInfo>, String> {
    let modules_path = std::path::Path::new(&project_path).join("modules");
    scanner::scan_modules_dir(&modules_path)
}

/// 扫描项目模块（多技术栈支持）
///
/// 根据技术栈类型调用对应的扫描策略，返回项目模块列表。
///
/// # 参数
/// - `project_path`: 项目根目录路径
/// - `tech_stack`: 技术栈类型标识（如 "fastapi"、"vue3"）
#[tauri::command]
pub async fn scan_project_modules(
    project_path: String,
    tech_stack: String,
) -> Result<Vec<ModuleInfo>, String> {
    let scanner = scan_strategy::get_scanner(&tech_stack)?;
    scanner.scan(std::path::Path::new(&project_path))
}
