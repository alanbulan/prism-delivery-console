// ============================================================================
// 打包服务：构建参数验证、目录复制、ZIP 打包
// 纯 Rust 函数，不依赖 tauri::*，方便单元测试
// ============================================================================

use std::io::Write;
use std::path::Path;

/// 验证构建参数：客户名称非空且至少选中一个模块
///
/// # 参数
/// - `client_name`: 客户名称
/// - `selected_modules`: 用户选中的模块列表
///
/// # 返回
/// - `Ok(())`: 验证通过
/// - `Err(String)`: 中文错误描述
pub fn validate_build_params(client_name: &str, selected_modules: &[String]) -> Result<(), String> {
    let name_empty = client_name.trim().is_empty();
    let no_modules = selected_modules.is_empty();

    match (name_empty, no_modules) {
        (true, true) => Err("构建验证失败：客户名称不能为空且至少需要选择一个模块".to_string()),
        (true, false) => Err("构建验证失败：客户名称不能为空".to_string()),
        (false, true) => Err("构建验证失败：至少需要选择一个模块".to_string()),
        (false, false) => Ok(()),
    }
}

/// 递归复制目录及其所有内容到目标路径
///
/// # 参数
/// - `src`: 源目录路径
/// - `dst`: 目标目录路径
///
/// # 返回
/// - `Ok(())`: 复制成功
/// - `Err(String)`: 中文错误描述
pub fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), String> {
    // 创建目标目录
    std::fs::create_dir_all(dst).map_err(|e| {
        format!("构建失败：复制文件时出错 - 无法创建目录 {}: {}", dst.display(), e)
    })?;

    // 使用 walkdir 遍历源目录
    for entry in walkdir::WalkDir::new(src) {
        let entry = entry.map_err(|e| {
            format!("构建失败：复制文件时出错 - 遍历目录失败: {}", e)
        })?;

        // 计算相对路径并拼接到目标路径
        let relative_path = entry.path().strip_prefix(src).map_err(|e| {
            format!("构建失败：复制文件时出错 - 路径处理失败: {}", e)
        })?;
        let target_path = dst.join(relative_path);

        if entry.file_type().is_dir() {
            // 创建对应的目录结构
            std::fs::create_dir_all(&target_path).map_err(|e| {
                format!("构建失败：复制文件时出错 - 无法创建目录 {}: {}", target_path.display(), e)
            })?;
        } else {
            // 复制文件
            std::fs::copy(entry.path(), &target_path).map_err(|e| {
                format!("构建失败：复制文件时出错 - 无法复制 {} 到 {}: {}",
                    entry.path().display(), target_path.display(), e)
            })?;
        }
    }

    Ok(())
}

/// 将目录内容打包为 ZIP 文件
///
/// # 参数
/// - `src_dir`: 要打包的源目录路径
/// - `zip_path`: 输出 ZIP 文件的路径
///
/// # 返回
/// - `Ok(())`: 打包成功
/// - `Err(String)`: 中文错误描述
pub fn create_zip_from_dir(src_dir: &Path, zip_path: &Path) -> Result<(), String> {
    // 创建 ZIP 文件
    let file = std::fs::File::create(zip_path).map_err(|e| {
        format!("构建失败：打包 ZIP 时出错 - 无法创建 ZIP 文件: {}", e)
    })?;
    let mut zip_writer = zip::ZipWriter::new(file);

    // 设置 ZIP 压缩选项（使用 Deflated 压缩）
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    // 使用 walkdir 遍历源目录中的所有文件
    for entry in walkdir::WalkDir::new(src_dir) {
        let entry = entry.map_err(|e| {
            format!("构建失败：打包 ZIP 时出错 - 遍历目录失败: {}", e)
        })?;

        let path = entry.path();
        // 计算相对于源目录的路径，作为 ZIP 内的文件名
        let relative_path = path.strip_prefix(src_dir).map_err(|e| {
            format!("构建失败：打包 ZIP 时出错 - 路径处理失败: {}", e)
        })?;

        // 跳过根目录本身
        if relative_path.as_os_str().is_empty() {
            continue;
        }

        // 统一使用正斜杠作为 ZIP 内路径分隔符
        let zip_entry_name = relative_path.to_string_lossy().replace('\\', "/");

        if path.is_dir() {
            // 添加目录条目（以 / 结尾）
            zip_writer
                .add_directory(format!("{}/", zip_entry_name), options)
                .map_err(|e| {
                    format!("构建失败：打包 ZIP 时出错 - 添加目录失败: {}", e)
                })?;
        } else {
            // 添加文件条目
            zip_writer
                .start_file(&zip_entry_name, options)
                .map_err(|e| {
                    format!("构建失败：打包 ZIP 时出错 - 添加文件失败: {}", e)
                })?;
            let content = std::fs::read(path).map_err(|e| {
                format!("构建失败：打包 ZIP 时出错 - 读取文件失败: {}", e)
            })?;
            zip_writer.write_all(&content).map_err(|e| {
                format!("构建失败：打包 ZIP 时出错 - 写入文件失败: {}", e)
            })?;
        }
    }

    // 完成 ZIP 写入
    zip_writer.finish().map_err(|e| {
        format!("构建失败：打包 ZIP 时出错 - 完成写入失败: {}", e)
    })?;

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
        assert!(result.unwrap_err().contains("客户名称不能为空"));
    }

    #[test]
    fn test_validate_build_params_whitespace_client_name() {
        let modules = vec!["auth".to_string()];
        let result = validate_build_params("   ", &modules);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("客户名称不能为空"));
    }

    #[test]
    fn test_validate_build_params_no_modules() {
        let modules: Vec<String> = vec![];
        let result = validate_build_params("客户A", &modules);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("至少需要选择一个模块"));
    }

    #[test]
    fn test_validate_build_params_both_invalid() {
        let modules: Vec<String> = vec![];
        let result = validate_build_params("", &modules);
        assert!(result.is_err());
        let err = result.unwrap_err();
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
