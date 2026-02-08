// ============================================================================
// 构建相关 Commands
// 负责：构建交付包（含多技术栈）、打开文件夹
// ============================================================================

use std::path::Path;

use crate::models::dtos::BuildResult;
use crate::services::packer::{copy_dir_recursive, create_zip_from_dir, validate_build_params};
use crate::services::build_strategy;
use crate::services::CORE_FILES;

/// 构建交付包：复制核心文件和选中模块，打包为 ZIP
///
/// 验证参数后，在项目目录下创建临时目录，复制核心文件和选中模块，
/// 打包为 ZIP 文件，最后清理临时目录。使用 scopeguard 确保清理。
#[tauri::command]
pub async fn build_package(
    project_path: String,
    selected_modules: Vec<String>,
    client_name: String,
) -> Result<BuildResult, String> {
    // 1. 验证构建参数
    validate_build_params(&client_name, &selected_modules)?;

    let project_dir = Path::new(&project_path);
    let dist_name = format!("dist_{}", client_name.trim());
    let temp_dir = project_dir.join(&dist_name);
    let zip_path = project_dir.join(format!("{}.zip", dist_name));

    // 2. 创建临时目录
    std::fs::create_dir_all(&temp_dir)
        .map_err(|_| "构建失败：无法创建临时目录".to_string())?;

    // 3. 使用 scopeguard 确保临时目录在任何情况下都会被清理
    let temp_dir_path = temp_dir.clone();
    let _guard = scopeguard::guard((), |_| {
        let _ = std::fs::remove_dir_all(&temp_dir_path);
    });

    // 4. 复制 Core_Files 白名单中的文件和目录
    for &core_item in CORE_FILES {
        let source = project_dir.join(core_item);
        if !source.exists() {
            continue;
        }

        if source.is_dir() {
            let dir_name = core_item.trim_end_matches('/');
            let dest = temp_dir.join(dir_name);
            copy_dir_recursive(&source, &dest)?;
        } else {
            let dest = temp_dir.join(core_item);
            std::fs::copy(&source, &dest).map_err(|e| {
                format!("构建失败：复制文件时出错 - 无法复制 {}: {}", core_item, e)
            })?;
        }
    }

    // 5. 创建 modules/ 子目录并复制选中的模块
    let modules_dest = temp_dir.join("modules");
    std::fs::create_dir_all(&modules_dest).map_err(|e| {
        format!("构建失败：复制文件时出错 - 无法创建 modules 目录: {}", e)
    })?;

    for module_name in &selected_modules {
        let module_src = project_dir.join("modules").join(module_name);
        let module_dst = modules_dest.join(module_name);

        if module_src.is_dir() {
            copy_dir_recursive(&module_src, &module_dst)?;
        }
    }

    // 6. 打包为 ZIP 文件
    create_zip_from_dir(&temp_dir, &zip_path)?;

    // 7. 返回构建结果
    let module_count = selected_modules.len();

    Ok(BuildResult {
        zip_path: zip_path.to_string_lossy().to_string(),
        client_name: client_name.trim().to_string(),
        module_count,
    })
}

/// 构建项目交付包（多技术栈支持）
///
/// 根据技术栈类型调用对应的构建策略。
/// 注意：此 command 不创建构建记录，前端应在构建成功后单独调用
/// db_create_build_record 来记录构建历史。
#[tauri::command]
pub async fn build_project_package(
    project_path: String,
    selected_modules: Vec<String>,
    client_name: String,
    tech_stack: String,
) -> Result<BuildResult, String> {
    let builder = build_strategy::get_builder(&tech_stack)?;
    builder.build(
        std::path::Path::new(&project_path),
        &selected_modules,
        &client_name,
    )
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
