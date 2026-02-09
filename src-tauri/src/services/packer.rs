// ============================================================================
// 打包服务：构建参数验证、目录复制、ZIP 打包
// 纯 Rust 函数，不依赖 tauri::*，方便单元测试
// ============================================================================

use std::io::{Read, Write};
use std::path::Path;

use crate::utils::error::{AppError, AppResult};

/// 验证构建参数：客户名称非空且至少选中一个模块
pub fn validate_build_params(client_name: &str, selected_modules: &[String]) -> AppResult<()> {
    let name_empty = client_name.trim().is_empty();
    let no_modules = selected_modules.is_empty();

    match (name_empty, no_modules) {
        (true, true) => Err(AppError::ValidationError(
            "客户名称不能为空且至少需要选择一个模块".to_string(),
        )),
        (true, false) => Err(AppError::ValidationError(
            "客户名称不能为空".to_string(),
        )),
        (false, true) => Err(AppError::ValidationError(
            "至少需要选择一个模块".to_string(),
        )),
        (false, false) => Ok(()),
    }
}

/// 递归复制目录及其所有内容到目标路径
pub fn copy_dir_recursive(src: &Path, dst: &Path) -> AppResult<()> {
    // 创建目标目录
    std::fs::create_dir_all(dst).map_err(|e| {
        AppError::BuildError(format!(
            "复制文件时出错 - 无法创建目录 {}: {}",
            dst.display(),
            e
        ))
    })?;

    // 使用 walkdir 遍历源目录
    for entry in walkdir::WalkDir::new(src) {
        let entry = entry.map_err(|e| {
            AppError::BuildError(format!("复制文件时出错 - 遍历目录失败: {}", e))
        })?;

        // 计算相对路径并拼接到目标路径
        let relative_path = entry
            .path()
            .strip_prefix(src)
            .map_err(|e| AppError::BuildError(format!("复制文件时出错 - 路径处理失败: {}", e)))?;
        let target_path = dst.join(relative_path);

        if entry.file_type().is_dir() {
            std::fs::create_dir_all(&target_path).map_err(|e| {
                AppError::BuildError(format!(
                    "复制文件时出错 - 无法创建目录 {}: {}",
                    target_path.display(),
                    e
                ))
            })?;
        } else {
            std::fs::copy(entry.path(), &target_path).map_err(|e| {
                AppError::BuildError(format!(
                    "复制文件时出错 - 无法复制 {} 到 {}: {}",
                    entry.path().display(),
                    target_path.display(),
                    e
                ))
            })?;
        }
    }

    Ok(())
}

/// 将目录内容打包为 ZIP 文件
pub fn create_zip_from_dir(src_dir: &Path, zip_path: &Path) -> AppResult<()> {
    let file = std::fs::File::create(zip_path)
        .map_err(|e| AppError::BuildError(format!("打包 ZIP 时出错 - 无法创建 ZIP 文件: {}", e)))?;
    let mut zip_writer = zip::ZipWriter::new(file);

    // 设置 ZIP 压缩选项（使用 Deflated 压缩）
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    for entry in walkdir::WalkDir::new(src_dir) {
        let entry = entry
            .map_err(|e| AppError::BuildError(format!("打包 ZIP 时出错 - 遍历目录失败: {}", e)))?;

        let path = entry.path();
        let relative_path = path
            .strip_prefix(src_dir)
            .map_err(|e| AppError::BuildError(format!("打包 ZIP 时出错 - 路径处理失败: {}", e)))?;

        // 跳过根目录本身
        if relative_path.as_os_str().is_empty() {
            continue;
        }

        // 统一使用正斜杠作为 ZIP 内路径分隔符
        let zip_entry_name = relative_path.to_string_lossy().replace('\\', "/");

        if path.is_dir() {
            zip_writer
                .add_directory(format!("{}/", zip_entry_name), options)
                .map_err(|e| AppError::BuildError(format!("打包 ZIP 时出错 - 添加目录失败: {}", e)))?;
        } else {
            zip_writer
                .start_file(&zip_entry_name, options)
                .map_err(|e| AppError::BuildError(format!("打包 ZIP 时出错 - 添加文件失败: {}", e)))?;
            // 流式写入：分块读取文件，避免大文件一次性加载到内存
            let mut file = std::fs::File::open(path)
                .map_err(|e| AppError::BuildError(format!("打包 ZIP 时出错 - 读取文件失败: {}", e)))?;
            let mut buf = [0u8; 64 * 1024]; // 64KB 缓冲区
            loop {
                let n = file.read(&mut buf)
                    .map_err(|e| AppError::BuildError(format!("打包 ZIP 时出错 - 读取文件失败: {}", e)))?;
                if n == 0 {
                    break;
                }
                zip_writer
                    .write_all(&buf[..n])
                    .map_err(|e| AppError::BuildError(format!("打包 ZIP 时出错 - 写入文件失败: {}", e)))?;
            }
        }
    }

    zip_writer
        .finish()
        .map_err(|e| AppError::BuildError(format!("打包 ZIP 时出错 - 完成写入失败: {}", e)))?;

    Ok(())
}
/// 复制项目目录到目标路径，排除指定的目录名
///
/// 用于构建时复制项目骨架：复制除 modules_dir 和忽略目录以外的所有文件。
/// 采用"排除法"替代"白名单法"，确保不遗漏任何核心文件。
///
/// # 参数
/// - `src`: 源项目根目录
/// - `dst`: 目标构建目录
/// - `exclude_dirs`: 需要排除的目录名列表（如 `[".git", "node_modules", "modules"]`）
pub fn copy_dir_excluding(src: &Path, dst: &Path, exclude_dirs: &[&str]) -> AppResult<()> {
    std::fs::create_dir_all(dst).map_err(|e| {
        AppError::BuildError(format!("无法创建目标目录 {}: {}", dst.display(), e))
    })?;

    for entry in walkdir::WalkDir::new(src)
        .into_iter()
        .filter_entry(|e| {
            // 只对目录做排除判断，文件始终保留
            if e.file_type().is_dir() {
                if let Some(name) = e.file_name().to_str() {
                    // 精确匹配或前缀匹配（如 "dist_" 匹配 "dist_客户A_20260209"）
                    for pattern in exclude_dirs {
                        if pattern.ends_with('_') {
                            // 前缀匹配模式
                            if name.starts_with(pattern) {
                                return false;
                            }
                        } else if pattern.starts_with("*.") {
                            // 通配符模式（如 "*.egg-info"）跳过，仅用于文件
                            continue;
                        } else if name == *pattern {
                            return false;
                        }
                    }
                }
            } else {
                // 文件级排除：处理通配符模式和精确文件名匹配
                if let Some(name) = e.file_name().to_str() {
                    for pattern in exclude_dirs {
                        if pattern.starts_with("*.") {
                            // 通配符后缀匹配（如 "*.egg-info"、"*.zip"）
                            let suffix = &pattern[1..]; // ".egg-info"
                            if name.ends_with(suffix) {
                                return false;
                            }
                        } else if pattern.starts_with('.') && name == *pattern {
                            // 精确匹配隐藏文件（如 ".env"、".env.local"）
                            return false;
                        }
                    }
                }
            }
            true
        })
    {
        let entry = entry.map_err(|e| {
            AppError::BuildError(format!("遍历项目目录失败: {}", e))
        })?;

        let relative = entry.path().strip_prefix(src).map_err(|e| {
            AppError::BuildError(format!("路径处理失败: {}", e))
        })?;

        // 跳过根目录本身
        if relative.as_os_str().is_empty() {
            continue;
        }

        let target = dst.join(relative);

        if entry.file_type().is_dir() {
            std::fs::create_dir_all(&target).map_err(|e| {
                AppError::BuildError(format!("无法创建目录 {}: {}", target.display(), e))
            })?;
        } else {
            // 确保父目录存在
            if let Some(parent) = target.parent() {
                std::fs::create_dir_all(parent).map_err(|e| {
                    AppError::BuildError(format!("无法创建目录 {}: {}", parent.display(), e))
                })?;
            }
            std::fs::copy(entry.path(), &target).map_err(|e| {
                AppError::BuildError(format!(
                    "无法复制 {} → {}: {}",
                    entry.path().display(),
                    target.display(),
                    e
                ))
            })?;
        }
    }

    Ok(())
}


// ============================================================================
// 单元测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_validate_build_params_valid() {
        let modules = vec!["auth".to_string()];
        let result = validate_build_params("客户A", &modules);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_build_params_empty_client_name() {
        let modules = vec!["auth".to_string()];
        let result = validate_build_params("", &modules);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("客户名称不能为空"));
    }

    #[test]
    fn test_validate_build_params_whitespace_client_name() {
        let modules = vec!["auth".to_string()];
        let result = validate_build_params("   ", &modules);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("客户名称不能为空"));
    }

    #[test]
    fn test_validate_build_params_no_modules() {
        let modules: Vec<String> = vec![];
        let result = validate_build_params("客户A", &modules);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("至少需要选择一个模块"));
    }

    #[test]
    fn test_validate_build_params_both_invalid() {
        let modules: Vec<String> = vec![];
        let result = validate_build_params("", &modules);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("客户名称不能为空"));
        assert!(err.contains("至少需要选择一个模块"));
    }

    #[test]
    fn test_copy_dir_recursive_basic() {
        let src_dir = TempDir::new().unwrap();
        let dst_dir = TempDir::new().unwrap();

        fs::write(src_dir.path().join("file1.txt"), "内容1").unwrap();
        fs::create_dir(src_dir.path().join("subdir")).unwrap();
        fs::write(src_dir.path().join("subdir").join("file2.txt"), "内容2").unwrap();

        let dest = dst_dir.path().join("copied");
        let result = copy_dir_recursive(src_dir.path(), &dest);
        assert!(result.is_ok());

        assert!(dest.join("file1.txt").exists());
        assert!(dest.join("subdir").join("file2.txt").exists());
        assert_eq!(fs::read_to_string(dest.join("file1.txt")).unwrap(), "内容1");
    }

    #[test]
    fn test_create_zip_from_dir_basic() {
        let dir = TempDir::new().unwrap();
        let src = dir.path().join("source");
        fs::create_dir(&src).unwrap();
        fs::write(src.join("hello.txt"), "你好世界").unwrap();
        fs::create_dir(src.join("sub")).unwrap();
        fs::write(src.join("sub").join("nested.txt"), "嵌套文件").unwrap();

        let zip_path = dir.path().join("output.zip");
        let result = create_zip_from_dir(&src, &zip_path);
        assert!(result.is_ok());
        assert!(zip_path.exists());

        // 解压验证内容
        let zip_file = fs::File::open(&zip_path).unwrap();
        let mut archive = zip::ZipArchive::new(zip_file).unwrap();
        let mut file_names: Vec<String> = Vec::new();
        for i in 0..archive.len() {
            let entry = archive.by_index(i).unwrap();
            if !entry.is_dir() {
                file_names.push(entry.name().to_string());
            }
        }
        assert!(file_names.contains(&"hello.txt".to_string()));
        assert!(file_names.contains(&"sub/nested.txt".to_string()));
    }
}
