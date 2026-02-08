// ============================================================================
// 构建相关 Commands
// 负责：构建交付包（含多技术栈）、打开文件夹
// ============================================================================

use crate::models::dtos::BuildResult;
use crate::services::build_strategy::{self, BuildStrategy};
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
    // 直接委托给 FastAPI 构建策略（V1 仅支持 FastAPI 项目，使用默认 modules 目录）
    let builder = build_strategy::FastApiBuildStrategy;
    builder.build(
        std::path::Path::new(&project_path),
        &selected_modules,
        &client_name,
        "",
    )
    .map_err(|e| e.to_string())
}

/// 构建项目交付包（多技术栈支持，带实时日志推送）
///
/// 根据技术栈类型调用对应的构建策略，通过 Tauri Event 向前端推送构建日志。
/// 注意：此 command 不创建构建记录，前端应在构建成功后单独调用
/// db_create_build_record 来记录构建历史。
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

    // 构建日志回调：通过 Tauri Event 推送到前端
    let log_fn = |msg: &str| {
        let _ = app.emit("build-log", msg.to_string());
    };

    builder.build_with_log(
        std::path::Path::new(&project_path),
        &selected_modules,
        &client_name,
        &modules_dir,
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
