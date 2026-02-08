// ============================================================================
// 多技术栈构建打包策略
// ============================================================================
//
// 使用策略模式（Strategy Pattern）实现可扩展的多技术栈构建打包。
// 每种技术栈实现 BuildStrategy trait，通过 get_builder 工厂函数获取对应策略。
// 新增技术栈只需添加新的 struct + impl，无需修改现有代码（OCP 原则）。

use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::models::dtos::BuildResult;
use crate::services::packer::{copy_dir_recursive, create_zip_from_dir, validate_build_params};
use crate::services::module_rewriter;
use crate::services::CORE_FILES;
use crate::utils::error::{AppError, AppResult};

// ============================================================================
// 构建策略 Trait 定义
// ============================================================================

/// 技术栈构建策略 trait
pub trait BuildStrategy {
    /// 该策略对应的技术栈标识（如 "fastapi"、"vue3"）
    fn tech_stack(&self) -> &str;

    /// 获取该技术栈的核心文件列表
    fn core_files(&self) -> &[&str];

    /// 获取模块所在的默认子目录名
    fn default_modules_dir(&self) -> &str;

    /// 执行构建打包
    /// - `modules_dir`: 用户自定义的模块目录（相对路径），为空则使用默认值
    fn build(
        &self,
        project_path: &Path,
        selected_modules: &[String],
        client_name: &str,
        modules_dir: &str,
    ) -> AppResult<BuildResult>;

    /// 执行构建打包（带日志回调，用于实时推送构建进度）
    fn build_with_log(
        &self,
        project_path: &Path,
        selected_modules: &[String],
        client_name: &str,
        modules_dir: &str,
        log_fn: &dyn Fn(&str),
    ) -> AppResult<BuildResult>;
}

// ============================================================================
// FastAPI 构建策略
// ============================================================================

/// FastAPI 构建策略：复制核心文件 + modules/ 子目录
pub struct FastApiBuildStrategy;

impl BuildStrategy for FastApiBuildStrategy {
    fn tech_stack(&self) -> &str {
        "fastapi"
    }

    fn core_files(&self) -> &[&str] {
        CORE_FILES
    }

    fn default_modules_dir(&self) -> &str {
        "modules"
    }

    fn build(
        &self,
        project_path: &Path,
        selected_modules: &[String],
        client_name: &str,
        modules_dir: &str,
    ) -> AppResult<BuildResult> {
        build_common(self, project_path, selected_modules, client_name, modules_dir)
    }

    fn build_with_log(
        &self,
        project_path: &Path,
        selected_modules: &[String],
        client_name: &str,
        modules_dir: &str,
        log_fn: &dyn Fn(&str),
    ) -> AppResult<BuildResult> {
        build_common_with_log(self, project_path, selected_modules, client_name, modules_dir, log_fn)
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
    fn tech_stack(&self) -> &str {
        "vue3"
    }

    fn core_files(&self) -> &[&str] {
        VUE3_CORE_FILES
    }

    fn default_modules_dir(&self) -> &str {
        "src/views"
    }

    fn build(
        &self,
        project_path: &Path,
        selected_modules: &[String],
        client_name: &str,
        modules_dir: &str,
    ) -> AppResult<BuildResult> {
        build_common(self, project_path, selected_modules, client_name, modules_dir)
    }

    fn build_with_log(
        &self,
        project_path: &Path,
        selected_modules: &[String],
        client_name: &str,
        modules_dir: &str,
        log_fn: &dyn Fn(&str),
    ) -> AppResult<BuildResult> {
        build_common_with_log(self, project_path, selected_modules, client_name, modules_dir, log_fn)
    }
}

// ============================================================================
// 通用构建流程（DRY 原则：提取公共逻辑）
// ============================================================================

/// 生成时间戳后缀（格式：yyyyMMdd_HHmmss）
fn timestamp_suffix() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    // 简易 UTC 时间格式化，避免引入 chrono 依赖（KISS 原则）
    let secs_per_day = 86400u64;
    let secs_per_hour = 3600u64;
    let secs_per_min = 60u64;

    let days = now / secs_per_day;
    let time_of_day = now % secs_per_day;
    let hour = time_of_day / secs_per_hour;
    let minute = (time_of_day % secs_per_hour) / secs_per_min;
    let second = time_of_day % secs_per_min;

    // 从 Unix 纪元天数计算年月日
    let (year, month, day) = days_to_ymd(days);
    format!("{:04}{:02}{:02}_{:02}{:02}{:02}", year, month, day, hour, minute, second)
}

/// 将 Unix 纪元天数转换为 (年, 月, 日)
fn days_to_ymd(days: u64) -> (u64, u64, u64) {
    // 简化的公历算法（足够满足时间戳需求）
    let mut y = 1970u64;
    let mut remaining = days;
    loop {
        let days_in_year = if is_leap(y) { 366 } else { 365 };
        if remaining < days_in_year {
            break;
        }
        remaining -= days_in_year;
        y += 1;
    }
    let months_days: [u64; 12] = if is_leap(y) {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };
    let mut m = 1u64;
    for &md in &months_days {
        if remaining < md {
            break;
        }
        remaining -= md;
        m += 1;
    }
    (y, m, remaining + 1)
}

fn is_leap(y: u64) -> bool {
    (y % 4 == 0 && y % 100 != 0) || y % 400 == 0
}

/// 带日志回调的通用构建流程
///
/// `log_fn` 在每个关键步骤完成时被调用，用于向前端推送实时日志。
/// 传入空闭包 `|_|{}` 即可静默执行（单元测试场景）。
pub fn build_common_with_log(
    strategy: &dyn BuildStrategy,
    project_path: &Path,
    selected_modules: &[String],
    client_name: &str,
    modules_dir_override: &str,
    log_fn: &dyn Fn(&str),
) -> AppResult<BuildResult> {
    // 1. 验证构建参数
    validate_build_params(client_name, selected_modules)?;
    log_fn("✓ 参数验证通过");

    // 用户自定义目录优先，为空则使用策略默认值
    let modules_dir_name = if modules_dir_override.is_empty() {
        strategy.default_modules_dir()
    } else {
        modules_dir_override
    };

    // 风险点1：路径含空格/特殊字符时记录警告（Rust Path API 本身可正确处理）
    let path_str = project_path.to_string_lossy();
    if path_str.contains(' ') || path_str.chars().any(|c| c > '\x7F') {
        log::warn!(
            "项目路径包含空格或非 ASCII 字符，可能影响部分外部工具兼容性: {}",
            path_str
        );
    }

    // 风险点4+5：使用时间戳后缀避免临时目录和 ZIP 文件名冲突
    let ts = timestamp_suffix();
    let dist_name = format!("dist_{}_{}", client_name.trim(), ts);
    let temp_dir = project_path.join(&dist_name);
    let zip_path = project_path.join(format!("{}.zip", dist_name));

    // 2. 创建临时目录
    std::fs::create_dir_all(&temp_dir)
        .map_err(|e| AppError::BuildError(format!("无法创建临时目录: {}", e)))?;
    log_fn(&format!("→ 创建临时目录: {}", dist_name));

    // 3. 使用 scopeguard 确保临时目录在任何情况下都会被清理
    let temp_dir_path = temp_dir.clone();
    let _guard = scopeguard::guard((), |_| {
        let _ = std::fs::remove_dir_all(&temp_dir_path);
    });

    // 4. 复制核心文件
    let core_files_list: Vec<&str> = strategy.core_files().iter()
        .filter(|&&f| project_path.join(f).exists())
        .copied()
        .collect();
    log_fn(&format!("→ 复制核心文件: {}", core_files_list.join(", ")));
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
    log_fn(&format!("✓ 核心文件复制完成 ({} 个)", core_files_list.len()));

    // 5. 创建模块子目录并复制选中的模块
    log_fn(&format!("→ 复制模块: {}", selected_modules.join(", ")));
    let modules_dest = temp_dir.join(modules_dir_name);
    std::fs::create_dir_all(&modules_dest)
        .map_err(|e| AppError::BuildError(format!("无法创建 {} 目录: {}", modules_dir_name, e)))?;

    let mut skipped_modules: Vec<String> = Vec::new();
    for module_name in selected_modules {
        let module_src = project_path.join(modules_dir_name).join(module_name);
        let module_dst = modules_dest.join(module_name);

        if module_src.is_dir() {
            copy_dir_recursive(&module_src, &module_dst)?;
            log_fn(&format!("  ✓ {}", module_name));
        } else {
            // 风险点6：模块目录不存在时记录警告而非静默跳过
            log::warn!(
                "选中的模块目录不存在，已跳过: {}",
                module_src.display()
            );
            skipped_modules.push(module_name.clone());
            log_fn(&format!("  ⚠ 跳过不存在的模块: {}", module_name));
        }
    }

    // 如果所有选中模块都不存在，视为构建失败
    if skipped_modules.len() == selected_modules.len() {
        return Err(AppError::BuildError(
            "所有选中的模块目录均不存在，无法构建".to_string(),
        ));
    }

    // 风险点3：大量文件时记录警告日志
    let file_count = walkdir::WalkDir::new(&temp_dir).into_iter().count();
    if file_count > 5000 {
        log::warn!(
            "构建包文件数量较多 ({} 个)，打包可能需要较长时间",
            file_count
        );
    }

    // 6. 重写入口文件中的模块导入（仅保留选中模块的 import 和 router 注册）
    // 根据技术栈获取对应的重写器，不支持的技术栈自动跳过
    if let Some(rewriter) = module_rewriter::get_rewriter(strategy.tech_stack()) {
        log_fn("→ 重写入口文件 import...");
        module_rewriter::process_entry_file(
            rewriter.as_ref(),
            &temp_dir,
            selected_modules,
            modules_dir_name,
        )?;
        log_fn("✓ import 重写完成");

        // 6.5 校验重写后的入口文件导入完整性
        log_fn("→ 校验导入完整性...");
        module_rewriter::validate_entry_file(
            rewriter.as_ref(),
            &temp_dir,
            modules_dir_name,
        )?;
        log_fn("✓ 导入校验通过");
    }

    // 7. 打包为 ZIP 文件
    log_fn(&format!("→ 打包 ZIP ({} 个文件)...", file_count));
    create_zip_from_dir(&temp_dir, &zip_path)?;
    log_fn("✓ ZIP 打包完成");

    // 8. 返回构建结果（实际打包的模块数 = 选中数 - 跳过数）
    let module_count = selected_modules.len() - skipped_modules.len();

    Ok(BuildResult {
        zip_path: zip_path.to_string_lossy().to_string(),
        client_name: client_name.trim().to_string(),
        module_count,
    })
}

/// 无日志版本的通用构建流程（向后兼容，供单元测试和不需要日志的场景使用）
fn build_common(
    strategy: &dyn BuildStrategy,
    project_path: &Path,
    selected_modules: &[String],
    client_name: &str,
    modules_dir_override: &str,
) -> AppResult<BuildResult> {
    build_common_with_log(strategy, project_path, selected_modules, client_name, modules_dir_override, &|_| {})
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
        let result = builder.build(dir.path(), &modules, "测试客户", "").unwrap();

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
        let result = builder.build(dir.path(), &modules, "客户B", "").unwrap();

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
        let result = builder.build(dir.path(), &modules, "", "");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("客户名称不能为空"));
    }

    #[test]
    fn test_build_with_no_modules_fails() {
        let dir = TempDir::new().unwrap();
        create_fastapi_project(&dir);

        let builder = FastApiBuildStrategy;
        let modules: Vec<String> = vec![];
        let result = builder.build(dir.path(), &modules, "客户A", "");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("至少需要选择一个模块"));
    }

    #[test]
    fn test_build_with_all_nonexistent_modules_fails() {
        let dir = TempDir::new().unwrap();
        create_fastapi_project(&dir);

        let builder = FastApiBuildStrategy;
        let modules = vec!["nonexistent_a".to_string(), "nonexistent_b".to_string()];
        let result = builder.build(dir.path(), &modules, "客户A", "");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("所有选中的模块目录均不存在"));
    }

    #[test]
    fn test_build_with_partial_nonexistent_modules_succeeds() {
        let dir = TempDir::new().unwrap();
        create_fastapi_project(&dir);

        let builder = FastApiBuildStrategy;
        // "auth" 存在，"nonexistent" 不存在
        let modules = vec!["auth".to_string(), "nonexistent".to_string()];
        let result = builder.build(dir.path(), &modules, "客户A", "").unwrap();
        // 实际打包的模块数应为 1（跳过了不存在的模块）
        assert_eq!(result.module_count, 1);
        assert_eq!(result.client_name, "客户A");

        let zip_path = Path::new(&result.zip_path);
        assert!(zip_path.exists());
        let _ = fs::remove_file(zip_path);
    }

    #[test]
    fn test_timestamp_suffix_format() {
        let ts = timestamp_suffix();
        // 格式应为 yyyyMMdd_HHmmss（15 个字符）
        assert_eq!(ts.len(), 15);
        assert_eq!(&ts[8..9], "_");
        // 所有非下划线字符应为数字
        assert!(ts.chars().filter(|&c| c != '_').all(|c| c.is_ascii_digit()));
    }

    #[test]
    fn test_zip_filename_contains_timestamp() {
        let dir = TempDir::new().unwrap();
        create_fastapi_project(&dir);

        let builder = FastApiBuildStrategy;
        let modules = vec!["auth".to_string()];
        let result = builder.build(dir.path(), &modules, "客户A", "").unwrap();

        // ZIP 路径应包含时间戳（dist_客户A_yyyyMMdd_HHmmss.zip）
        assert!(result.zip_path.contains("dist_客户A_"));
        // 文件名中应有 15 位时间戳
        let filename = Path::new(&result.zip_path)
            .file_stem()
            .unwrap()
            .to_string_lossy()
            .to_string();
        assert!(filename.starts_with("dist_客户A_"));

        let _ = fs::remove_file(&result.zip_path);
    }
}
