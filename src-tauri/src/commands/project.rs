// ============================================================================
// 项目相关 Commands
// 负责：项目打开、模块扫描（含多技术栈）
// ============================================================================

use crate::models::dtos::{ModuleInfo, ProjectInfo};
use crate::services::scan_strategy;
use crate::services::scanner;

/// 打开项目：弹出原生文件夹选择对话框，返回项目路径
///
/// 调用系统原生文件夹选择对话框，用户选择文件夹后返回路径。
/// 不再强制验证 FastAPI 项目结构，由前端根据技术栈选择后调用对应扫描策略。
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

    Ok(ProjectInfo {
        path: path.to_string_lossy().to_string(),
        core_files: vec![], // 核心文件由前端选择技术栈后通过扫描获取
    })
}

/// 扫描模块：读取 modules/ 下的一级子目录，过滤忽略项
///
/// 接收项目路径，拼接 modules/ 子目录后调用 services 层执行扫描。
#[tauri::command]
pub async fn scan_modules(project_path: String) -> Result<Vec<ModuleInfo>, String> {
    let modules_path = std::path::Path::new(&project_path).join("modules");
    scanner::scan_modules_dir(&modules_path).map_err(|e| e.to_string())
}

/// 扫描项目模块（多技术栈支持）
///
/// 根据技术栈类型调用对应的扫描策略，返回项目模块列表。
///
/// # 参数
/// - `project_path`: 项目根目录路径
/// - `tech_stack`: 技术栈类型标识（如 "fastapi"、"vue3"）
/// - `modules_dir`: 用户自定义的模块目录（相对路径），为空则使用技术栈默认值
#[tauri::command]
pub async fn scan_project_modules(
    project_path: String,
    tech_stack: String,
    modules_dir: String,
) -> Result<Vec<ModuleInfo>, String> {
    let scanner = scan_strategy::get_scanner(&tech_stack).map_err(|e| e.to_string())?;
    scanner.scan(std::path::Path::new(&project_path), &modules_dir).map_err(|e| e.to_string())
}
