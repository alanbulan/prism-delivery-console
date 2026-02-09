// ============================================================================
// 构建相关 Commands
// 负责：构建交付包（含多技术栈）、打开文件夹
// ============================================================================

use crate::models::dtos::BuildResult;
use crate::services::build_strategy::{self, BuildStrategy};
use crate::services::scanner;
use tauri::Emitter;

/// 构建交付包（V1 兼容接口）：委托给 FastAPI 构建策略
///
/// 此命令保留为 QuickBuildPage 的后端接口，内部直接复用
/// FastApiBuildStrategy 的构建逻辑，消除重复代码。
#[tauri::command]
pub async fn build_package(
    project_path: String,
    selected_modules: Vec<String>,
    client_name: String,
) -> Result<BuildResult, String> {
    let path = std::path::Path::new(&project_path);
    // 扫描所有模块名用于依赖分析
    let all_module_names: Vec<String> = scanner::scan_modules_dir(&path.join("modules"))
        .unwrap_or_default()
        .into_iter()
        .map(|m| m.name)
        .collect();

    let builder = build_strategy::FastApiBuildStrategy;
    builder.build(
        path,
        &selected_modules,
        &client_name,
        "",
        &all_module_names,
    )
    .map_err(|e| e.to_string())
}

/// 构建项目交付包（多技术栈支持，带实时日志推送）
///
/// 根据技术栈类型调用对应的构建策略，通过 Tauri Event 向前端推送构建日志。
/// 构建前自动扫描所有模块名，用于 BFS 传递依赖分析。
#[tauri::command]
pub async fn build_project_package(
    app: tauri::AppHandle,
    project_path: String,
    selected_modules: Vec<String>,
    client_name: String,
    tech_stack: String,
    modules_dir: String,
) -> Result<BuildResult, String> {
    let builder = build_strategy::get_builder(&tech_stack).map_err(|e| e.to_string())?;
    let path = std::path::Path::new(&project_path);

    // 确定模块目录（用户自定义优先，否则使用策略默认值）
    let modules_dir_name = if modules_dir.is_empty() {
        builder.default_modules_dir()
    } else {
        &modules_dir
    };

    // 扫描所有模块名用于依赖分析
    let all_module_names: Vec<String> = scanner::scan_modules_dir(&path.join(modules_dir_name))
        .unwrap_or_default()
        .into_iter()
        .map(|m| m.name)
        .collect();

    // 构建日志回调：通过 Tauri Event 推送到前端
    let log_fn = |msg: &str| {
        let _ = app.emit("build-log", msg.to_string());
    };

    builder.build_with_log(
        path,
        &selected_modules,
        &client_name,
        &modules_dir,
        &all_module_names,
        &log_fn,
    )
    .map_err(|e| e.to_string())
}

/// 打开文件夹：在系统文件管理器中打开指定路径（并选中该文件）
#[tauri::command]
pub async fn open_folder(path: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer.exe")
            .arg("/select,")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("打开文件夹失败：无法启动资源管理器 - {}", e))?;
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg("-R")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("打开文件夹失败：无法启动 Finder - {}", e))?;
    }

    #[cfg(target_os = "linux")]
    {
        let file_path = std::path::Path::new(&path);
        let parent_dir = file_path
            .parent()
            .ok_or_else(|| "打开文件夹失败：无法获取文件所在目录".to_string())?;
        std::process::Command::new("xdg-open")
            .arg(parent_dir)
            .spawn()
            .map_err(|e| format!("打开文件夹失败：无法启动文件管理器 - {}", e))?;
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        return Err("打开文件夹失败：不支持当前操作系统".to_string());
    }

    Ok(())
}
