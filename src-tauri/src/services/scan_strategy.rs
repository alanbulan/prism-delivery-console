// ============================================================================
// 多技术栈模块扫描策略
// ============================================================================
//
// 使用策略模式（Strategy Pattern）实现可扩展的多技术栈模块扫描。
// 每种技术栈实现 ScanStrategy trait，通过 get_scanner 工厂函数获取对应策略。
// 新增技术栈只需添加新的 struct + impl，无需修改现有代码（OCP 原则）。

use std::path::Path;

use crate::models::dtos::ModuleInfo;
use crate::utils::error::{AppError, AppResult};

// ============================================================================
// 扫描策略 Trait 定义
// ============================================================================

/// 技术栈扫描策略 trait
pub trait ScanStrategy {
    /// 扫描项目模块，返回模块列表
    fn scan(&self, project_path: &Path) -> AppResult<Vec<ModuleInfo>>;
}

// ============================================================================
// FastAPI 扫描策略
// ============================================================================

/// FastAPI 扫描策略：扫描 modules/ 子目录
///
/// 复用 `services::scanner::scan_modules_dir` 逻辑，扫描项目根目录下的 modules/ 目录，
/// 返回一级子目录作为模块列表，自动过滤 __pycache__、.git 等忽略条目。
pub struct FastApiScanner;

impl ScanStrategy for FastApiScanner {
    fn scan(&self, project_path: &Path) -> AppResult<Vec<ModuleInfo>> {
        let modules_dir = project_path.join("modules");
        if !modules_dir.is_dir() {
            return Err(AppError::ScanError(
                "fastapi 项目应包含 modules 目录".to_string(),
            ));
        }
        crate::services::scanner::scan_modules_dir(&modules_dir)
    }
}

// ============================================================================
// Vue3 扫描策略（复用 scan_modules_dir，消除重复代码）
// ============================================================================

/// Vue3 扫描策略：扫描 src/views/ 子目录
pub struct Vue3Scanner;

impl ScanStrategy for Vue3Scanner {
    fn scan(&self, project_path: &Path) -> AppResult<Vec<ModuleInfo>> {
        let views_dir = project_path.join("src").join("views");
        if !views_dir.is_dir() {
            return Err(AppError::ScanError(
                "vue3 项目应包含 src/views 目录".to_string(),
            ));
        }
        // 复用通用扫描逻辑，消除与 FastApiScanner 的重复代码（P3.7 优化）
        crate::services::scanner::scan_modules_dir(&views_dir)
    }
}

// ============================================================================
// 工厂函数
// ============================================================================

/// 根据技术栈类型获取对应的扫描策略
pub fn get_scanner(tech_stack: &str) -> AppResult<Box<dyn ScanStrategy>> {
    match tech_stack {
        "fastapi" => Ok(Box::new(FastApiScanner)),
        "vue3" => Ok(Box::new(Vue3Scanner)),
        _ => Err(AppError::UnsupportedTechStack(tech_stack.to_string())),
    }
}


// ============================================================================
// 单元测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_fastapi_project(dir: &TempDir, module_names: &[&str]) {
        let modules_dir = dir.path().join("modules");
        std::fs::create_dir_all(&modules_dir).unwrap();
        for name in module_names {
            std::fs::create_dir_all(modules_dir.join(name)).unwrap();
        }
    }

    fn create_vue3_project(dir: &TempDir, view_names: &[&str]) {
        let views_dir = dir.path().join("src").join("views");
        std::fs::create_dir_all(&views_dir).unwrap();
        for name in view_names {
            std::fs::create_dir_all(views_dir.join(name)).unwrap();
        }
    }

    #[test]
    fn test_fastapi_scanner_scans_modules_correctly() {
        let dir = TempDir::new().unwrap();
        create_fastapi_project(&dir, &["auth", "users", "orders"]);

        let scanner = FastApiScanner;
        let result = scanner.scan(dir.path()).unwrap();
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].name, "auth");
        assert_eq!(result[1].name, "orders");
        assert_eq!(result[2].name, "users");
    }

    #[test]
    fn test_fastapi_scanner_filters_ignored_entries() {
        let dir = TempDir::new().unwrap();
        create_fastapi_project(&dir, &["auth", "__pycache__", ".git", ".DS_Store"]);

        let scanner = FastApiScanner;
        let result = scanner.scan(dir.path()).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "auth");
    }

    #[test]
    fn test_fastapi_scanner_missing_modules_dir() {
        let dir = TempDir::new().unwrap();
        let scanner = FastApiScanner;
        let result = scanner.scan(dir.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_vue3_scanner_scans_views_correctly() {
        let dir = TempDir::new().unwrap();
        create_vue3_project(&dir, &["dashboard", "login", "settings"]);

        let scanner = Vue3Scanner;
        let result = scanner.scan(dir.path()).unwrap();
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].name, "dashboard");
        assert_eq!(result[1].name, "login");
        assert_eq!(result[2].name, "settings");
    }

    #[test]
    fn test_vue3_scanner_missing_views_dir() {
        let dir = TempDir::new().unwrap();
        let scanner = Vue3Scanner;
        let result = scanner.scan(dir.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_get_scanner_fastapi() {
        let scanner = get_scanner("fastapi");
        assert!(scanner.is_ok());
    }

    #[test]
    fn test_get_scanner_vue3() {
        let scanner = get_scanner("vue3");
        assert!(scanner.is_ok());
    }

    #[test]
    fn test_get_scanner_unsupported() {
        let result = get_scanner("django");
        match result {
            Err(err) => assert!(err.to_string().contains("不支持的技术栈类型")),
            Ok(_) => panic!("应返回错误，但返回了 Ok"),
        }
    }
}
