// ============================================================================
// 多技术栈构建打包策略
// ============================================================================
//
// 使用策略模式（Strategy Pattern）实现可扩展的多技术栈构建打包。
// 每种技术栈实现 BuildStrategy trait，通过 get_builder 工厂函数获取对应策略。
// 新增技术栈只需添加新的 struct + impl，无需修改现有代码（OCP 原则）。

use std::path::Path;

use time::OffsetDateTime;

use crate::models::dtos::BuildResult;
use crate::services::analyzer;
use crate::services::packer::{copy_dir_excluding, create_zip_from_dir, validate_build_params};
use crate::services::module_rewriter;
use crate::services::DEFAULT_EXCLUDES;
use crate::utils::error::{AppError, AppResult};

// ============================================================================
// 构建策略 Trait 定义
// ============================================================================

/// 技术栈构建策略 trait
pub trait BuildStrategy {
    /// 该策略对应的技术栈标识（如 "fastapi"、"vue3"）
    fn tech_stack(&self) -> &str;

    /// 获取该技术栈额外需要排除的目录（在 DEFAULT_EXCLUDES 基础上追加）
    fn extra_excludes(&self) -> Vec<String>;

    /// 获取模块所在的默认子目录名
    fn default_modules_dir(&self) -> &str;

    /// 执行构建打包
    /// - `modules_dir`: 用户自定义的模块目录（相对路径），为空则使用默认值
    /// - `all_module_names`: 项目中所有可用模块名（用于依赖分析）
    fn build(
        &self,
        project_path: &Path,
        selected_modules: &[String],
        client_name: &str,
        modules_dir: &str,
        all_module_names: &[String],
    ) -> AppResult<BuildResult>;

    /// 执行构建打包（带日志回调，用于实时推送构建进度）
    fn build_with_log(
        &self,
        project_path: &Path,
        selected_modules: &[String],
        client_name: &str,
        modules_dir: &str,
        all_module_names: &[String],
        log_fn: &dyn Fn(&str),
    ) -> AppResult<BuildResult>;
}

// ============================================================================
// FastAPI 构建策略
// ============================================================================

/// FastAPI 构建策略：排除式骨架复制 + 依赖分析 + 模块提取
pub struct FastApiBuildStrategy;

impl BuildStrategy for FastApiBuildStrategy {
    fn tech_stack(&self) -> &str {
        "fastapi"
    }

    fn extra_excludes(&self) -> Vec<String> {
        // FastAPI 项目无额外排除项（DEFAULT_EXCLUDES 已覆盖）
        vec![]
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
        all_module_names: &[String],
    ) -> AppResult<BuildResult> {
        build_common(self, project_path, selected_modules, client_name, modules_dir, all_module_names)
    }

    fn build_with_log(
        &self,
        project_path: &Path,
        selected_modules: &[String],
        client_name: &str,
        modules_dir: &str,
        all_module_names: &[String],
        log_fn: &dyn Fn(&str),
    ) -> AppResult<BuildResult> {
        build_common_with_log(self, project_path, selected_modules, client_name, modules_dir, all_module_names, log_fn)
    }
}


// ============================================================================
// Vue3 构建策略
// ============================================================================

/// Vue3 构建策略：排除式骨架复制 + 依赖分析 + 模块提取
pub struct Vue3BuildStrategy;

impl BuildStrategy for Vue3BuildStrategy {
    fn tech_stack(&self) -> &str {
        "vue3"
    }

    fn extra_excludes(&self) -> Vec<String> {
        // Vue3 项目无额外排除项（DEFAULT_EXCLUDES 已覆盖）
        vec![]
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
        all_module_names: &[String],
    ) -> AppResult<BuildResult> {
        build_common(self, project_path, selected_modules, client_name, modules_dir, all_module_names)
    }

    fn build_with_log(
        &self,
        project_path: &Path,
        selected_modules: &[String],
        client_name: &str,
        modules_dir: &str,
        all_module_names: &[String],
        log_fn: &dyn Fn(&str),
    ) -> AppResult<BuildResult> {
        build_common_with_log(self, project_path, selected_modules, client_name, modules_dir, all_module_names, log_fn)
    }
}

// ============================================================================
// 通用构建流程（DRY 原则：提取公共逻辑）
// ============================================================================

/// 生成时间戳后缀（格式：yyyyMMdd_HHmmss）
///
/// 使用 `time` crate 替代手写日历算法，更可靠且可维护（KISS 原则）
fn timestamp_suffix() -> String {
    let now = OffsetDateTime::now_utc();
    format!(
        "{:04}{:02}{:02}_{:02}{:02}{:02}",
        now.year(),
        now.month() as u8,
        now.day(),
        now.hour(),
        now.minute(),
        now.second()
    )
}

/// 获取指定路径所在磁盘的可用空间（字节）
///
/// 使用 Windows GetDiskFreeSpaceExW API。失败时返回 0（不阻断构建）。
#[cfg(target_os = "windows")]
fn fs_available_space(path: &Path) -> u64 {
    use std::os::windows::ffi::OsStrExt;
    // 将路径转为宽字符（UTF-16），以 null 结尾
    let wide: Vec<u16> = path.as_os_str().encode_wide().chain(std::iter::once(0)).collect();
    let mut free_bytes: u64 = 0;
    // 安全：调用 Windows API，传入有效的 null 结尾宽字符串
    unsafe {
        extern "system" {
            fn GetDiskFreeSpaceExW(
                lpDirectoryName: *const u16,
                lpFreeBytesAvailableToCaller: *mut u64,
                lpTotalNumberOfBytes: *mut u64,
                lpTotalNumberOfFreeBytes: *mut u64,
            ) -> i32;
        }
        GetDiskFreeSpaceExW(wide.as_ptr(), &mut free_bytes, std::ptr::null_mut(), std::ptr::null_mut());
    }
    free_bytes
}

/// 非 Windows 平台的磁盘空间检查（返回 0 跳过检查）
#[cfg(not(target_os = "windows"))]
fn fs_available_space(_path: &Path) -> u64 {
    0
}

/// 带日志回调的通用构建流程（V2：排除式骨架 + 依赖分析）
///
/// 构建流程：
/// 1. 复制项目骨架（排除模块目录 + DEFAULT_EXCLUDES + 技术栈额外排除项）
/// 2. 依赖分析：BFS 遍历选中模块的 import，自动补充被依赖的模块
/// 3. 复制扩展后的完整模块列表到骨架中
/// 4. 重写入口文件（仅保留选中+依赖模块的 import）
/// 5. 打包为 ZIP
pub fn build_common_with_log(
    strategy: &dyn BuildStrategy,
    project_path: &Path,
    selected_modules: &[String],
    client_name: &str,
    modules_dir_override: &str,
    all_module_names: &[String],
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

    // 路径含空格/特殊字符时记录警告
    let path_str = project_path.to_string_lossy();
    if path_str.contains(' ') || path_str.chars().any(|c| c > '\x7F') {
        log::warn!(
            "项目路径包含空格或非 ASCII 字符，可能影响部分外部工具兼容性: {}",
            path_str
        );
    }

    // 时间戳后缀避免临时目录和 ZIP 文件名冲突
    let ts = timestamp_suffix();
    let dist_name = format!("dist_{}_{}", client_name.trim(), ts);
    let temp_dir = project_path.join(&dist_name);
    let zip_path = project_path.join(format!("{}.zip", dist_name));

    // 磁盘空间预检：确保可用空间 > 项目目录大小的 2 倍（骨架复制 + ZIP 打包）
    if let Ok(entries) = std::fs::read_dir(project_path) {
        // 快速估算项目大小（仅统计一级目录，避免深度遍历耗时）
        let estimated_size: u64 = entries
            .filter_map(|e| e.ok())
            .filter_map(|e| e.metadata().ok())
            .map(|m| m.len())
            .sum();
        // 使用 Windows API 获取磁盘可用空间
        let available = fs_available_space(project_path);
        if available > 0 && estimated_size > 0 && available < estimated_size * 2 {
            return Err(AppError::BuildError(format!(
                "磁盘可用空间不足：需要约 {} MB，当前可用 {} MB",
                estimated_size * 2 / 1024 / 1024,
                available / 1024 / 1024
            )));
        }
    }

    // 2. 创建临时目录
    std::fs::create_dir_all(&temp_dir)
        .map_err(|e| AppError::BuildError(format!("无法创建临时目录: {}", e)))?;
    log_fn(&format!("→ 创建临时目录: {}", dist_name));

    // scopeguard 确保临时目录在任何情况下都会被清理
    let temp_dir_path = temp_dir.clone();
    let _guard = scopeguard::guard((), |_| {
        let _ = std::fs::remove_dir_all(&temp_dir_path);
    });

    // 3. 排除式骨架复制：复制整个项目，排除默认排除项 + 技术栈额外排除项
    //    这样 main.py、config/、utils/、package.json、src/router/ 等全部自动包含
    let mut exclude_list: Vec<&str> = DEFAULT_EXCLUDES.to_vec();
    // 排除 dist_ 开头的临时目录和 ZIP 文件
    exclude_list.push("dist_");
    exclude_list.push("*.zip");
    // 追加技术栈额外排除项（先存储 owned 值，再借用引用）
    let extra = strategy.extra_excludes();
    for ex in &extra {
        exclude_list.push(ex.as_str());
    }

    log_fn(&format!("→ 复制项目骨架（排除 {} 项噪音目录）...", exclude_list.len()));
    copy_dir_excluding(project_path, &temp_dir, &exclude_list)?;

    // 删除骨架中的模块目录内容（后续单独复制选中的模块）
    let skeleton_modules_dir = temp_dir.join(modules_dir_name);
    // 先备份 modules/__init__.py（如果存在），避免 remove_dir_all 后丢失包初始化逻辑
    let init_py_backup = skeleton_modules_dir.join("__init__.py");
    let init_py_content = if init_py_backup.exists() {
        std::fs::read_to_string(&init_py_backup).ok()
    } else {
        None
    };
    if skeleton_modules_dir.is_dir() {
        std::fs::remove_dir_all(&skeleton_modules_dir)
            .map_err(|e| AppError::BuildError(format!("清理模块目录失败: {}", e)))?;
    }
    log_fn("✓ 项目骨架复制完成");

    // 4. 依赖分析：BFS 遍历选中模块的 import，自动补充被依赖的模块
    log_fn(&format!("→ 依赖分析：选中模块 [{}]", selected_modules.join(", ")));
    let (expanded_modules, auto_added) = if all_module_names.is_empty() {
        // 没有提供全部模块名时跳过依赖分析（向后兼容）
        log_fn("  ⚠ 未提供模块列表，跳过依赖分析");
        (selected_modules.to_vec(), Vec::new())
    } else {
        match analyzer::resolve_module_dependencies(
            project_path,
            modules_dir_name,
            selected_modules,
            all_module_names,
        ) {
            Ok((full_list, added)) => {
                if !added.is_empty() {
                    log_fn(&format!("  → 自动补充依赖模块: [{}]", added.join(", ")));
                }
                (full_list, added)
            }
            Err(e) => {
                // 依赖分析失败不阻断构建，降级为仅复制选中模块
                log_fn(&format!("  ⚠ 依赖分析失败（{}），仅复制选中模块", e));
                (selected_modules.to_vec(), Vec::new())
            }
        }
    };
    log_fn(&format!(
        "✓ 依赖分析完成：共 {} 个模块（选中 {} + 自动补充 {}）",
        expanded_modules.len(),
        selected_modules.len(),
        auto_added.len()
    ));

    // 5. 创建模块子目录并复制扩展后的模块列表
    log_fn(&format!("→ 复制模块: {}", expanded_modules.join(", ")));
    let modules_dest = temp_dir.join(modules_dir_name);
    std::fs::create_dir_all(&modules_dest)
        .map_err(|e| AppError::BuildError(format!("无法创建 {} 目录: {}", modules_dir_name, e)))?;

    // 恢复 modules/__init__.py（Python 包初始化文件，可能包含 __all__ 等配置）
    if let Some(ref content) = init_py_content {
        std::fs::write(modules_dest.join("__init__.py"), content)
            .map_err(|e| AppError::BuildError(format!("恢复 __init__.py 失败: {}", e)))?;
        log_fn("  ✓ 已恢复 __init__.py");
    }

    let mut skipped_modules: Vec<String> = Vec::new();
    for module_name in &expanded_modules {
        let module_src = project_path.join(modules_dir_name).join(module_name);
        let module_dst = modules_dest.join(module_name);

        if module_src.is_dir() {
            crate::services::packer::copy_dir_recursive(&module_src, &module_dst)?;
            let tag = if auto_added.contains(module_name) { " (依赖)" } else { "" };
            log_fn(&format!("  ✓ {}{}", module_name, tag));
        } else {
            log::warn!("选中的模块目录不存在，已跳过: {}", module_src.display());
            skipped_modules.push(module_name.clone());
            log_fn(&format!("  ⚠ 跳过不存在的模块: {}", module_name));
        }
    }

    // 如果所有模块都不存在，视为构建失败
    if skipped_modules.len() == expanded_modules.len() {
        return Err(AppError::BuildError(
            "所有选中的模块目录均不存在，无法构建".to_string(),
        ));
    }

    // 大量文件时记录警告日志
    let file_count = walkdir::WalkDir::new(&temp_dir).into_iter().count();
    if file_count > 5000 {
        log::warn!(
            "构建包文件数量较多 ({} 个)，打包可能需要较长时间",
            file_count
        );
    }

    // 6. 重写入口文件中的模块导入（仅保留扩展后模块列表的 import 和 router 注册）
    if let Some(rewriter) = module_rewriter::get_rewriter(strategy.tech_stack()) {
        log_fn("→ 重写入口文件 import...");
        module_rewriter::process_entry_file(
            rewriter.as_ref(),
            &temp_dir,
            &expanded_modules,
            modules_dir_name,
        )?;
        log_fn("✓ import 重写完成");

        // 校验重写后的入口文件导入完整性
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

    // 8. 返回构建结果（实际打包的模块数 = 扩展后总数 - 跳过数）
    let module_count = expanded_modules.len() - skipped_modules.len();
    // 过滤掉跳过的模块，返回实际打包的完整模块列表
    let actual_modules: Vec<String> = expanded_modules
        .into_iter()
        .filter(|m| !skipped_modules.contains(m))
        .collect();

    Ok(BuildResult {
        zip_path: zip_path.to_string_lossy().to_string(),
        client_name: client_name.trim().to_string(),
        module_count,
        expanded_modules: actual_modules,
    })
}

/// 无日志版本的通用构建流程（向后兼容，供单元测试和不需要日志的场景使用）
fn build_common(
    strategy: &dyn BuildStrategy,
    project_path: &Path,
    selected_modules: &[String],
    client_name: &str,
    modules_dir_override: &str,
    all_module_names: &[String],
) -> AppResult<BuildResult> {
    build_common_with_log(strategy, project_path, selected_modules, client_name, modules_dir_override, all_module_names, &|_| {})
}

// ============================================================================
// 工厂函数
// ============================================================================

/// 根据技术栈类型获取对应的构建策略
///
/// 优先匹配内置策略（fastapi/vue3），未匹配时尝试从数据库加载自定义模板
pub fn get_builder(tech_stack: &str) -> AppResult<Box<dyn BuildStrategy>> {
    match tech_stack {
        "fastapi" => Ok(Box::new(FastApiBuildStrategy)),
        "vue3" => Ok(Box::new(Vue3BuildStrategy)),
        _ => Err(AppError::UnsupportedTechStack(tech_stack.to_string())),
    }
}

/// 根据数据库模板配置获取通用构建策略
///
/// 供 commands 层在查到自定义模板后调用，避免 services 层直接依赖 Database
pub fn get_generic_builder(
    name: String,
    modules_dir: String,
    extra_excludes_json: String,
) -> AppResult<Box<dyn BuildStrategy>> {
    // 解析额外排除目录 JSON 数组
    let extra: Vec<String> = serde_json::from_str(&extra_excludes_json).unwrap_or_default();
    Ok(Box::new(GenericBuildStrategy {
        name,
        modules_dir,
        extra_excludes: extra,
    }))
}

// ============================================================================
// 通用构建策略（基于数据库模板配置）
// ============================================================================

/// 通用构建策略：从数据库模板读取配置，适用于用户自定义的技术栈
pub struct GenericBuildStrategy {
    /// 技术栈名称
    name: String,
    /// 模块扫描目录
    modules_dir: String,
    /// 额外排除目录列表
    extra_excludes: Vec<String>,
}

impl BuildStrategy for GenericBuildStrategy {
    fn tech_stack(&self) -> &str {
        &self.name
    }

    fn extra_excludes(&self) -> Vec<String> {
        // 返回用户自定义的额外排除目录
        self.extra_excludes.clone()
    }

    fn default_modules_dir(&self) -> &str {
        &self.modules_dir
    }

    fn build(
        &self,
        project_path: &Path,
        selected_modules: &[String],
        client_name: &str,
        modules_dir: &str,
        all_module_names: &[String],
    ) -> AppResult<BuildResult> {
        build_common(self, project_path, selected_modules, client_name, modules_dir, all_module_names)
    }

    fn build_with_log(
        &self,
        project_path: &Path,
        selected_modules: &[String],
        client_name: &str,
        modules_dir: &str,
        all_module_names: &[String],
        log_fn: &dyn Fn(&str),
    ) -> AppResult<BuildResult> {
        build_common_with_log(self, project_path, selected_modules, client_name, modules_dir, all_module_names, log_fn)
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
        let all_modules = vec!["auth".to_string(), "billing".to_string(), "users".to_string()];
        let result = builder.build(dir.path(), &modules, "测试客户", "", &all_modules).unwrap();

        assert_eq!(result.client_name, "测试客户");
        assert_eq!(result.module_count, 2);

        let zip_path = Path::new(&result.zip_path);
        assert!(zip_path.exists());

        let entries = read_zip_entries(zip_path);
        // 骨架复制应包含 main.py、config/、core/、utils/ 等
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
        let all_modules = vec!["dashboard".to_string(), "login".to_string()];
        let result = builder.build(dir.path(), &modules, "客户B", "", &all_modules).unwrap();

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
    }

    #[test]
    fn test_get_builder_vue3() {
        let builder = get_builder("vue3");
        assert!(builder.is_ok());
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
        let result = builder.build(dir.path(), &modules, "", "", &[]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("客户名称不能为空"));
    }

    #[test]
    fn test_build_with_no_modules_fails() {
        let dir = TempDir::new().unwrap();
        create_fastapi_project(&dir);

        let builder = FastApiBuildStrategy;
        let modules: Vec<String> = vec![];
        let result = builder.build(dir.path(), &modules, "客户A", "", &[]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("至少需要选择一个模块"));
    }

    #[test]
    fn test_build_with_all_nonexistent_modules_fails() {
        let dir = TempDir::new().unwrap();
        create_fastapi_project(&dir);

        let builder = FastApiBuildStrategy;
        let modules = vec!["nonexistent_a".to_string(), "nonexistent_b".to_string()];
        let result = builder.build(dir.path(), &modules, "客户A", "", &[]);
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
        let all_modules = vec!["auth".to_string(), "billing".to_string(), "users".to_string()];
        let result = builder.build(dir.path(), &modules, "客户A", "", &all_modules).unwrap();
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
        let all_modules = vec!["auth".to_string(), "billing".to_string(), "users".to_string()];
        let result = builder.build(dir.path(), &modules, "客户A", "", &all_modules).unwrap();

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
