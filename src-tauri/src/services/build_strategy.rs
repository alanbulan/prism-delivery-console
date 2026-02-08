// ============================================================================
// 多技术栈构建打包策略
// ============================================================================
//
// 使用策略模式（Strategy Pattern）实现可扩展的多技术栈构建打包。
// 每种技术栈实现 BuildStrategy trait，通过 get_builder 工厂函数获取对应策略。
// 新增技术栈只需添加新的 struct + impl，无需修改现有代码（OCP 原则）。

use std::path::Path;

use crate::models::dtos::BuildResult;
use crate::services::packer::{copy_dir_recursive, create_zip_from_dir, validate_build_params};
use crate::services::CORE_FILES;
use crate::utils::error::{AppError, AppResult};

// ============================================================================
// 构建策略 Trait 定义
// ============================================================================

/// 技术栈构建策略 trait
pub trait BuildStrategy {
    /// 获取该技术栈的核心文件列表
    fn core_files(&self) -> &[&str];

    /// 获取模块所在的子目录名
    fn modules_dir(&self) -> &str;

    /// 执行构建打包
    fn build(
        &self,
        project_path: &Path,
        selected_modules: &[String],
        client_name: &str,
    ) -> AppResult<BuildResult>;
}

// ============================================================================
// FastAPI 构建策略
// ============================================================================

/// FastAPI 构建策略：复制核心文件 + modules/ 子目录
pub struct FastApiBuildStrategy;

impl BuildStrategy for FastApiBuildStrategy {
    fn core_files(&self) -> &[&str] {
        CORE_FILES
    }

    fn modules_dir(&self) -> &str {
        "modules"
    }

    fn build(
        &self,
        project_path: &Path,
        selected_modules: &[String],
        client_name: &str,
    ) -> AppResult<BuildResult> {
        build_common(self, project_path, selected_modules, client_name)
    }
}


// ============================================================================
// Vue3 构建策略
// ============================================================================

/// Vue3 核心配置文件列表
const VUE3_CORE_FILES: &[&str] = &[
    "package.json",
    "vite.config.ts",
    "tsconfig.json",
    "index.html",
];

/// Vue3 构建策略：复制项目配置文件 + src/views/ 子目录
pub struct Vue3BuildStrategy;

impl BuildStrategy for Vue3BuildStrategy {
    fn core_files(&self) -> &[&str] {
        VUE3_CORE_FILES
    }

    fn modules_dir(&self) -> &str {
        "src/views"
    }

    fn build(
        &self,
        project_path: &Path,
        selected_modules: &[String],
        client_name: &str,
    ) -> AppResult<BuildResult> {
        build_common(self, project_path, selected_modules, client_name)
    }
}

// ============================================================================
// 通用构建流程（DRY 原则：提取公共逻辑）
// ============================================================================

/// 通用构建流程，供各策略复用
fn build_common(
    strategy: &dyn BuildStrategy,
    project_path: &Path,
    selected_modules: &[String],
    client_name: &str,
) -> AppResult<BuildResult> {
    // 1. 验证构建参数
    validate_build_params(client_name, selected_modules)?;

    let dist_name = format!("dist_{}", client_name.trim());
    let temp_dir = project_path.join(&dist_name);
    let zip_path = project_path.join(format!("{}.zip", dist_name));

    // 2. 创建临时目录
    std::fs::create_dir_all(&temp_dir)
        .map_err(|e| AppError::BuildError(format!("无法创建临时目录: {}", e)))?;

    // 3. 使用 scopeguard 确保临时目录在任何情况下都会被清理
    let temp_dir_path = temp_dir.clone();
    let _guard = scopeguard::guard((), |_| {
        let _ = std::fs::remove_dir_all(&temp_dir_path);
    });

    // 4. 复制核心文件
    for &core_item in strategy.core_files() {
        let source = project_path.join(core_item);
        if !source.exists() {
            continue;
        }

        if source.is_dir() {
            let dir_name = core_item.trim_end_matches('/');
            let dest = temp_dir.join(dir_name);
            copy_dir_recursive(&source, &dest)?;
        } else {
            let dest = temp_dir.join(core_item);
            // 确保父目录存在（处理嵌套路径如 "src/views"）
            if let Some(parent) = dest.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| AppError::BuildError(format!("无法创建目录 {}: {}", parent.display(), e)))?;
            }
            std::fs::copy(&source, &dest)
                .map_err(|e| AppError::BuildError(format!("复制文件时出错 - 无法复制 {}: {}", core_item, e)))?;
        }
    }

    // 5. 创建模块子目录并复制选中的模块
    let modules_dir_name = strategy.modules_dir();
    let modules_dest = temp_dir.join(modules_dir_name);
    std::fs::create_dir_all(&modules_dest)
        .map_err(|e| AppError::BuildError(format!("无法创建 {} 目录: {}", modules_dir_name, e)))?;

    for module_name in selected_modules {
        let module_src = project_path.join(modules_dir_name).join(module_name);
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

// ============================================================================
// 工厂函数
// ============================================================================

/// 根据技术栈类型获取对应的构建策略
pub fn get_builder(tech_stack: &str) -> AppResult<Box<dyn BuildStrategy>> {
    match tech_stack {
        "fastapi" => Ok(Box::new(FastApiBuildStrategy)),
        "vue3" => Ok(Box::new(Vue3BuildStrategy)),
        _ => Err(AppError::UnsupportedTechStack(tech_stack.to_string())),
    }
}


// ============================================================================
// 单元测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_fastapi_project(dir: &TempDir) {
        let root = dir.path();
        fs::write(root.join("main.py"), "# FastAPI main").unwrap();
        fs::write(root.join("requirements.txt"), "fastapi").unwrap();
        fs::write(root.join(".env.example"), "SECRET=xxx").unwrap();
        fs::create_dir_all(root.join("config")).unwrap();
        fs::write(root.join("config").join("settings.py"), "# 配置").unwrap();
        fs::create_dir_all(root.join("core")).unwrap();
        fs::write(root.join("core").join("base.py"), "# 核心").unwrap();
        fs::create_dir_all(root.join("utils")).unwrap();
        fs::write(root.join("utils").join("helpers.py"), "# 工具").unwrap();
        fs::create_dir_all(root.join("modules").join("auth")).unwrap();
        fs::write(root.join("modules").join("auth").join("routes.py"), "# 认证").unwrap();
        fs::create_dir_all(root.join("modules").join("billing")).unwrap();
        fs::write(root.join("modules").join("billing").join("routes.py"), "# 计费").unwrap();
        fs::create_dir_all(root.join("modules").join("users")).unwrap();
        fs::write(root.join("modules").join("users").join("routes.py"), "# 用户").unwrap();
    }

    fn create_vue3_project(dir: &TempDir) {
        let root = dir.path();
        fs::write(root.join("package.json"), r#"{"name":"test"}"#).unwrap();
        fs::write(root.join("vite.config.ts"), "export default {}").unwrap();
        fs::write(root.join("tsconfig.json"), r#"{"compilerOptions":{}}"#).unwrap();
        fs::write(root.join("index.html"), "<html></html>").unwrap();
        fs::create_dir_all(root.join("src").join("views").join("dashboard")).unwrap();
        fs::write(
            root.join("src").join("views").join("dashboard").join("index.vue"),
            "<template>Dashboard</template>",
        ).unwrap();
        fs::create_dir_all(root.join("src").join("views").join("login")).unwrap();
        fs::write(
            root.join("src").join("views").join("login").join("index.vue"),
            "<template>Login</template>",
        ).unwrap();
    }

    fn read_zip_entries(zip_path: &Path) -> Vec<String> {
        let file = fs::File::open(zip_path).unwrap();
        let mut archive = zip::ZipArchive::new(file).unwrap();
        let mut entries = Vec::new();
        for i in 0..archive.len() {
            let entry = archive.by_index(i).unwrap();
            entries.push(entry.name().to_string());
        }
        entries
    }

    #[test]
    fn test_fastapi_build_produces_correct_zip() {
        let dir = TempDir::new().unwrap();
        create_fastapi_project(&dir);

        let builder = FastApiBuildStrategy;
        let modules = vec!["auth".to_string(), "users".to_string()];
        let result = builder.build(dir.path(), &modules, "测试客户").unwrap();

        assert_eq!(result.client_name, "测试客户");
        assert_eq!(result.module_count, 2);

        let zip_path = Path::new(&result.zip_path);
        assert!(zip_path.exists());

        let entries = read_zip_entries(zip_path);
        assert!(entries.iter().any(|n| n == "main.py"));
        assert!(entries.iter().any(|n| n.starts_with("modules/auth")));
        assert!(entries.iter().any(|n| n.starts_with("modules/users")));
        assert!(!entries.iter().any(|n| n.starts_with("modules/billing")));

        let _ = fs::remove_file(zip_path);
    }

    #[test]
    fn test_vue3_build_produces_correct_zip() {
        let dir = TempDir::new().unwrap();
        create_vue3_project(&dir);

        let builder = Vue3BuildStrategy;
        let modules = vec!["dashboard".to_string(), "login".to_string()];
        let result = builder.build(dir.path(), &modules, "客户B").unwrap();

        assert_eq!(result.client_name, "客户B");
        assert_eq!(result.module_count, 2);

        let zip_path = Path::new(&result.zip_path);
        assert!(zip_path.exists());

        let entries = read_zip_entries(zip_path);
        assert!(entries.iter().any(|n| n == "package.json"));
        assert!(entries.iter().any(|n| n.starts_with("src/views/dashboard")));
        assert!(entries.iter().any(|n| n.starts_with("src/views/login")));

        let _ = fs::remove_file(zip_path);
    }

    #[test]
    fn test_get_builder_fastapi() {
        let builder = get_builder("fastapi");
        assert!(builder.is_ok());
        assert!(builder.unwrap().core_files().contains(&"main.py"));
    }

    #[test]
    fn test_get_builder_vue3() {
        let builder = get_builder("vue3");
        assert!(builder.is_ok());
        assert!(builder.unwrap().core_files().contains(&"package.json"));
    }

    #[test]
    fn test_get_builder_unsupported() {
        let result = get_builder("django");
        assert!(result.is_err());
    }

    #[test]
    fn test_build_with_empty_client_name_fails() {
        let dir = TempDir::new().unwrap();
        create_fastapi_project(&dir);

        let builder = FastApiBuildStrategy;
        let modules = vec!["auth".to_string()];
        let result = builder.build(dir.path(), &modules, "");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("客户名称不能为空"));
    }

    #[test]
    fn test_build_with_no_modules_fails() {
        let dir = TempDir::new().unwrap();
        create_fastapi_project(&dir);

        let builder = FastApiBuildStrategy;
        let modules: Vec<String> = vec![];
        let result = builder.build(dir.path(), &modules, "客户A");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("至少需要选择一个模块"));
    }
}
