// ============================================================================
// 扫描服务：项目验证与模块扫描
// 纯 Rust 函数，不依赖 tauri::*，方便单元测试
// ============================================================================

use crate::models::dtos::ModuleInfo;
use crate::services::{CORE_FILES, IGNORED_ENTRIES};
use crate::utils::error::{AppError, AppResult};

/// 验证项目文件夹结构并扫描核心文件
///
/// 检查指定路径下是否包含 `main.py` 文件和 `modules/` 目录，
/// 并扫描核心文件白名单中实际存在的文件/目录。
pub fn validate_project(path: &std::path::Path) -> AppResult<Vec<String>> {
    let has_main_py = path.join("main.py").exists();
    let has_modules = path.join("modules").is_dir();

    match (has_main_py, has_modules) {
        (false, false) => {
            return Err(AppError::ValidationError(
                "缺少 main.py 文件和 modules/ 目录".to_string(),
            ));
        }
        (false, true) => {
            return Err(AppError::ValidationError("缺少 main.py 文件".to_string()));
        }
        (true, false) => {
            return Err(AppError::ValidationError("缺少 modules/ 目录".to_string()));
        }
        (true, true) => {} // 验证通过
    }

    // 扫描核心文件白名单中实际存在的文件/目录
    let core_files: Vec<String> = CORE_FILES
        .iter()
        .filter(|&name| {
            let full_path = path.join(name);
            if name.ends_with('/') {
                full_path.is_dir()
            } else {
                full_path.exists()
            }
        })
        .map(|&name| name.to_string())
        .collect();

    Ok(core_files)
}

/// 扫描 modules 目录下的一级子目录，过滤忽略条目
pub fn scan_modules_dir(modules_path: &std::path::Path) -> AppResult<Vec<ModuleInfo>> {
    let entries = std::fs::read_dir(modules_path)
        .map_err(|_| AppError::ScanError("无法读取 modules/ 目录".to_string()))?;

    let mut modules: Vec<ModuleInfo> = entries
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false))
        .filter(|entry| {
            let name = entry.file_name().to_string_lossy().to_string();
            !IGNORED_ENTRIES.contains(&name.as_str())
        })
        .map(|entry| {
            let name = entry.file_name().to_string_lossy().to_string();
            let path = entry.path().to_string_lossy().to_string();
            ModuleInfo { name, path }
        })
        .collect();

    modules.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(modules)
}


// ============================================================================
// 单元测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_valid_project(dir: &TempDir) {
        fs::write(dir.path().join("main.py"), "# FastAPI main").unwrap();
        fs::create_dir(dir.path().join("modules")).unwrap();
    }

    #[test]
    fn test_validate_project_valid_minimal() {
        let dir = TempDir::new().unwrap();
        create_valid_project(&dir);

        let result = validate_project(dir.path());
        assert!(result.is_ok());
        let core_files = result.unwrap();
        assert!(core_files.contains(&"main.py".to_string()));
    }

    #[test]
    fn test_validate_project_missing_main_py() {
        let dir = TempDir::new().unwrap();
        fs::create_dir(dir.path().join("modules")).unwrap();

        let result = validate_project(dir.path());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("缺少 main.py 文件"));
    }

    #[test]
    fn test_validate_project_missing_modules() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("main.py"), "# main").unwrap();

        let result = validate_project(dir.path());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("缺少 modules/ 目录"));
    }

    #[test]
    fn test_validate_project_missing_both() {
        let dir = TempDir::new().unwrap();

        let result = validate_project(dir.path());
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("缺少 main.py 文件"));
        assert!(err.contains("modules/ 目录"));
    }

    #[test]
    fn test_validate_project_scans_core_files() {
        let dir = TempDir::new().unwrap();
        create_valid_project(&dir);
        fs::write(dir.path().join("requirements.txt"), "fastapi").unwrap();
        fs::create_dir(dir.path().join("config")).unwrap();

        let result = validate_project(dir.path()).unwrap();
        assert!(result.contains(&"main.py".to_string()));
        assert!(result.contains(&"requirements.txt".to_string()));
        assert!(result.contains(&"config/".to_string()));
        assert!(!result.contains(&".env.example".to_string()));
    }

    #[test]
    fn test_validate_project_all_core_files_present() {
        let dir = TempDir::new().unwrap();
        create_valid_project(&dir);
        fs::write(dir.path().join("requirements.txt"), "").unwrap();
        fs::write(dir.path().join(".env.example"), "").unwrap();
        fs::create_dir(dir.path().join("config")).unwrap();
        fs::create_dir(dir.path().join("core")).unwrap();
        fs::create_dir(dir.path().join("utils")).unwrap();

        let result = validate_project(dir.path()).unwrap();
        assert_eq!(result.len(), CORE_FILES.len());
    }

    #[test]
    fn test_scan_modules_dir_empty() {
        let dir = TempDir::new().unwrap();
        let modules_path = dir.path().join("modules");
        fs::create_dir(&modules_path).unwrap();

        let result = scan_modules_dir(&modules_path).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_scan_modules_dir_normal_modules() {
        let dir = TempDir::new().unwrap();
        let modules_path = dir.path().join("modules");
        fs::create_dir(&modules_path).unwrap();
        fs::create_dir(modules_path.join("auth")).unwrap();
        fs::create_dir(modules_path.join("billing")).unwrap();
        fs::create_dir(modules_path.join("users")).unwrap();

        let result = scan_modules_dir(&modules_path).unwrap();
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].name, "auth");
        assert_eq!(result[1].name, "billing");
        assert_eq!(result[2].name, "users");
    }

    #[test]
    fn test_scan_modules_dir_filters_ignored_entries() {
        let dir = TempDir::new().unwrap();
        let modules_path = dir.path().join("modules");
        fs::create_dir(&modules_path).unwrap();
        fs::create_dir(modules_path.join("auth")).unwrap();
        fs::create_dir(modules_path.join("__pycache__")).unwrap();
        fs::create_dir(modules_path.join(".git")).unwrap();

        let result = scan_modules_dir(&modules_path).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "auth");
    }

    #[test]
    fn test_scan_modules_dir_nonexistent_path() {
        let dir = TempDir::new().unwrap();
        let nonexistent = dir.path().join("nonexistent");

        let result = scan_modules_dir(&nonexistent);
        assert!(result.is_err());
    }
}
