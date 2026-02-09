// ============================================================================
// 模块导入重写器（策略模式）
// ============================================================================
//
// 构建交付包时，自动处理入口文件中的模块导入/注册代码。
// 根据用户选中的模块列表，移除未选中模块的相关行，确保交付包能直接启动。
//
// 使用 ImportRewriter trait 实现可扩展的多技术栈支持：
// - FastApiImportRewriter: 处理 main.py 中的 from modules.xxx import / app.include_router
// - Vue3ImportRewriter: 处理 router/index.ts 中的 import / route 定义（预留）
//
// 新增技术栈只需实现 ImportRewriter trait，无需修改现有代码（OCP 原则）。
// ============================================================================

use std::collections::{HashMap, HashSet};
use std::path::Path;

use crate::utils::error::{AppError, AppResult};

// ============================================================================
// ImportRewriter Trait 定义
// ============================================================================

/// 模块导入重写策略 trait
pub trait ImportRewriter {
    /// 入口文件的相对路径（如 "main.py"、"src/router/index.ts"）
    fn entry_file(&self) -> &str;

    /// 重写入口文件内容，只保留选中模块的导入和注册
    ///
    /// # 参数
    /// - `content`: 入口文件原始内容
    /// - `selected_modules`: 用户选中的模块名列表
    /// - `modules_dir`: 模块目录名（如 "modules"、"src/views"）
    fn rewrite(
        &self,
        content: &str,
        selected_modules: &[String],
        modules_dir: &str,
    ) -> String;

    /// 校验重写后的入口文件中，所有模块导入引用的路径在构建目录中是否存在
    ///
    /// 返回缺失的模块路径列表。空列表 = 校验通过。
    /// 如果返回非空，说明源项目代码本身存在问题（引用了不存在的模块）。
    fn validate(
        &self,
        content: &str,
        build_dir: &Path,
        modules_dir: &str,
    ) -> Vec<String>;
}

/// 在构建目录中执行入口文件重写
///
/// 读取入口文件 → 调用 rewriter 重写 → 覆盖写回。
/// 如果入口文件不存在则跳过（不报错）。
pub fn process_entry_file(
    rewriter: &dyn ImportRewriter,
    build_dir: &Path,
    selected_modules: &[String],
    modules_dir: &str,
) -> AppResult<()> {
    let entry_path = build_dir.join(rewriter.entry_file());
    if !entry_path.exists() {
        log::warn!(
            "构建目录中未找到入口文件 {}，跳过模块导入重写",
            rewriter.entry_file()
        );
        return Ok(());
    }

    let content = std::fs::read_to_string(&entry_path).map_err(|e| {
        AppError::BuildError(format!("读取 {} 失败：{}", rewriter.entry_file(), e))
    })?;

    let rewritten = rewriter.rewrite(&content, selected_modules, modules_dir);

    std::fs::write(&entry_path, rewritten).map_err(|e| {
        AppError::BuildError(format!("写入 {} 失败：{}", rewriter.entry_file(), e))
    })?;

    log::info!(
        "已重写 {} 模块导入：保留 {} 个模块",
        rewriter.entry_file(),
        selected_modules.len()
    );

    Ok(())
}

/// 校验构建目录中入口文件的导入完整性
///
/// 读取重写后的入口文件，调用 rewriter.validate() 检查所有模块导入
/// 引用的路径是否在构建目录中实际存在。
/// 如果存在缺失导入，返回 BuildError。
pub fn validate_entry_file(
    rewriter: &dyn ImportRewriter,
    build_dir: &Path,
    modules_dir: &str,
) -> AppResult<()> {
    let entry_path = build_dir.join(rewriter.entry_file());
    if !entry_path.exists() {
        // 入口文件不存在则跳过校验（与 process_entry_file 行为一致）
        return Ok(());
    }

    let content = std::fs::read_to_string(&entry_path).map_err(|e| {
        AppError::BuildError(format!("校验时读取 {} 失败：{}", rewriter.entry_file(), e))
    })?;

    let missing = rewriter.validate(&content, build_dir, modules_dir);
    if !missing.is_empty() {
        return Err(AppError::BuildError(format!(
            "导入完整性校验失败：以下模块在构建目录中不存在 → {}",
            missing.join(", ")
        )));
    }

    Ok(())
}

// ============================================================================
// FastAPI 导入重写器
// ============================================================================

/// FastAPI 导入重写器
///
/// 处理 main.py 中的模块导入，支持 3 种主流 import 模式：
/// 1. `from modules.xxx.routes import router as xxx_router`
/// 2. `from modules.xxx import routes as xxx_routes`
/// 3. `from modules import xxx, yyy`
pub struct FastApiImportRewriter;

impl ImportRewriter for FastApiImportRewriter {
    fn entry_file(&self) -> &str {
        "main.py"
    }

    fn rewrite(
        &self,
        content: &str,
        selected_modules: &[String],
        modules_dir: &str,
    ) -> String {
        rewrite_python_imports(content, selected_modules, modules_dir)
    }

    fn validate(
        &self,
        content: &str,
        build_dir: &Path,
        modules_dir: &str,
    ) -> Vec<String> {
        validate_python_imports(content, build_dir, modules_dir)
    }
}

// ============================================================================
// Vue3 导入重写器
// ============================================================================

/// Vue3 导入重写器
///
/// 处理 router/index.ts 中的路由导入和注册，支持 3 种主流模式：
///
/// **模式 1：静态导入**
/// ```ts
/// import DashboardView from '@/views/dashboard/index.vue'
/// ```
/// → 移除未选中模块的 import 行 + 对应路由对象
///
/// **模式 2：动态懒加载**
/// ```ts
/// component: () => import('@/views/dashboard/index.vue')
/// ```
/// → 移除包含未选中模块路径的路由对象（含花括号块）
///
/// **模式 3：自动路由（unplugin-vue-router / vite-plugin-pages）**
/// → 路由由文件系统自动生成，无需重写入口文件。
///    构建时只需确保 modules_dir 中仅包含选中模块的目录即可。
pub struct Vue3ImportRewriter;

impl ImportRewriter for Vue3ImportRewriter {
    fn entry_file(&self) -> &str {
        "src/router/index.ts"
    }

    fn rewrite(
        &self,
        content: &str,
        selected_modules: &[String],
        modules_dir: &str,
    ) -> String {
        rewrite_vue3_router(content, selected_modules, modules_dir)
    }

    fn validate(
        &self,
        content: &str,
        build_dir: &Path,
        modules_dir: &str,
    ) -> Vec<String> {
        validate_vue3_imports(content, build_dir, modules_dir)
    }
}

// ============================================================================
// 工厂函数
// ============================================================================

/// 根据技术栈获取对应的导入重写器
///
/// 返回 None 表示该技术栈不需要导入重写
pub fn get_rewriter(tech_stack: &str) -> Option<Box<dyn ImportRewriter>> {
    match tech_stack {
        "fastapi" => Some(Box::new(FastApiImportRewriter)),
        "vue3" => Some(Box::new(Vue3ImportRewriter)),
        _ => None,
    }
}

/// 根据数据库模板配置获取通用导入重写器
///
/// 当模板的 entry_file 和 import_pattern 均非空时返回 Some，否则返回 None（跳过重写）
pub fn get_generic_rewriter(
    entry_file: String,
    import_pattern: String,
    router_pattern: String,
) -> Option<Box<dyn ImportRewriter>> {
    if entry_file.is_empty() || import_pattern.is_empty() {
        return None; // 未配置入口文件或导入模式，跳过重写
    }
    Some(Box::new(GenericImportRewriter {
        entry_file,
        import_pattern,
        _router_pattern: router_pattern,
    }))
}

// ============================================================================
// 通用导入重写器（基于正则模式匹配）
// ============================================================================

/// 通用导入重写器：使用用户配置的正则表达式匹配模块导入
///
/// import_pattern 中的 `{modules_dir}` 占位符会在运行时替换为实际模块目录。
/// 正则的第一个捕获组应为模块名。
pub struct GenericImportRewriter {
    entry_file: String,
    import_pattern: String,
    _router_pattern: String,
}

impl ImportRewriter for GenericImportRewriter {
    fn entry_file(&self) -> &str {
        &self.entry_file
    }

    fn rewrite(
        &self,
        content: &str,
        selected_modules: &[String],
        modules_dir: &str,
    ) -> String {
        // 将 {modules_dir} 占位符替换为实际值，构建正则
        let pattern_str = self.import_pattern.replace("{modules_dir}", modules_dir);
        let re = match regex::Regex::new(&pattern_str) {
            Ok(r) => r,
            Err(_) => return content.to_string(), // 正则无效，原样返回
        };

        let selected: std::collections::HashSet<&str> =
            selected_modules.iter().map(|s| s.as_str()).collect();

        // 逐行过滤：匹配到模块导入且模块名不在选中列表中 → 移除
        content
            .lines()
            .filter(|line| {
                if let Some(caps) = re.captures(line) {
                    if let Some(module_name) = caps.get(1) {
                        return selected.contains(module_name.as_str());
                    }
                }
                true // 非模块导入行 → 保留
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn validate(
        &self,
        _content: &str,
        _build_dir: &Path,
        _modules_dir: &str,
    ) -> Vec<String> {
        // 通用重写器暂不做深度校验，返回空列表表示通过
        Vec::new()
    }
}

// ============================================================================
// Vue3 路由重写核心逻辑（供 Vue3ImportRewriter 使用）
// ============================================================================

/// 将 modules_dir 转换为 Vue3 import 路径中的别名前缀
///
/// 例如：
/// - "src/views" → "@/views" （标准 @ 别名）
/// - "views" → "@/views"（假设在 src/ 下）
/// - "src/pages" → "@/pages"
fn to_vue3_import_prefix(modules_dir: &str) -> String {
    // 去掉开头的 "src/"，因为 Vue3 项目中 @ 别名通常指向 src/
    let stripped = modules_dir.strip_prefix("src/").unwrap_or(modules_dir);
    format!("@/{}", stripped)
}

/// 从 Vue3 import 路径中提取模块名（views 目录下的第一级子目录）
///
/// 例如：
/// - `@/views/dashboard/index.vue` → Some("dashboard")
/// - `@/views/system/user/index.vue` → Some("system")
/// - `@/components/Button.vue` → None（不在 views 目录下）
/// - `../views/login/index.vue` → Some("login")（相对路径）
fn extract_vue3_module_name(import_path: &str, import_prefix: &str) -> Option<String> {
    // 尝试匹配 @/views/xxx 或自定义前缀
    let after_prefix = if let Some(rest) = import_path.strip_prefix(import_prefix) {
        rest.strip_prefix('/')
    } else {
        None
    };

    let after_prefix = after_prefix?;

    // 取第一个 "/" 之前的部分作为模块名
    let module_name = match after_prefix.find('/') {
        Some(pos) => &after_prefix[..pos],
        None => after_prefix.trim_end_matches(".vue").trim_end_matches(".ts"),
    };

    if module_name.is_empty() {
        return None;
    }

    Some(module_name.to_string())
}

/// 重写 Vue3 router/index.ts 文件，只保留选中模块的路由
///
/// 处理两种主流模式：
/// 1. 静态 import + routes 数组中引用
/// 2. 动态 import() 内联在 routes 数组中
///
/// 策略：
/// - 第一遍：过滤顶层 import 行，收集被移除的 import 标识符
/// - 第二遍：过滤 routes 数组中引用了未选中模块的路由对象（花括号块）
fn rewrite_vue3_router(
    content: &str,
    selected_modules: &[String],
    modules_dir: &str,
) -> String {
    let selected: HashSet<&str> = selected_modules.iter().map(|s| s.as_str()).collect();
    let import_prefix = to_vue3_import_prefix(modules_dir);

    let lines: Vec<&str> = content.lines().collect();
    let mut output: Vec<String> = Vec::new();

    // 收集被移除的静态 import 标识符（用于后续过滤路由对象）
    let mut removed_identifiers: HashSet<String> = HashSet::new();

    // ---- 第一遍：逐行处理 import 语句和路由对象 ----
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim();

        // 处理静态 import 语句：import XxxView from '@/views/xxx/...'
        if let Some((identifier, module_name)) =
            parse_static_import(trimmed, &import_prefix)
        {
            if selected.contains(module_name.as_str()) {
                output.push(line.to_string());
            } else {
                // 未选中 → 移除此行，记录标识符
                removed_identifiers.insert(identifier);
            }
            i += 1;
            continue;
        }

        // 处理 const Xxx = () => import('...') 形式的顶层懒加载声明
        if let Some((identifier, module_name)) =
            parse_lazy_const_import(trimmed, &import_prefix)
        {
            if selected.contains(module_name.as_str()) {
                output.push(line.to_string());
            } else {
                removed_identifiers.insert(identifier);
            }
            i += 1;
            continue;
        }

        // 处理路由对象块 { ... }（可能跨多行）
        // 检测是否是路由对象的开始（以 { 开头，在数组上下文中）
        if is_route_object_start(trimmed) {
            // 收集整个路由对象块
            let (block_lines, end_idx) = collect_brace_block(&lines, i);
            let block_text = block_lines.join("\n");

            // 判断此路由对象是否应被移除
            if should_remove_route_block(
                &block_text,
                &selected,
                &removed_identifiers,
                &import_prefix,
            ) {
                // 跳过整个块
                i = end_idx + 1;
                continue;
            }

            // 保留整个块
            for li in i..=end_idx {
                if li < lines.len() {
                    output.push(lines[li].to_string());
                }
            }
            i = end_idx + 1;
            continue;
        }

        // 其他行 → 原样保留
        output.push(line.to_string());
        i += 1;
    }

    output.join("\n")
}

/// 解析静态 import 语句，返回 (标识符, 模块名)
///
/// 匹配模式：`import XxxView from '@/views/xxx/...'`
fn parse_static_import(line: &str, import_prefix: &str) -> Option<(String, String)> {
    // 必须以 "import " 开头（排除 "import {" 和 "import type"）
    if !line.starts_with("import ") {
        return None;
    }

    let after_import = line.strip_prefix("import ")?.trim_start();

    // 排除 `import { xxx }` 和 `import type` 形式
    if after_import.starts_with('{') || after_import.starts_with("type ") {
        return None;
    }

    // 查找 " from " 分隔符
    let from_pos = after_import.find(" from ")?;
    let identifier = after_import[..from_pos].trim().to_string();
    let path_part = after_import[from_pos + 6..].trim();

    // 提取引号内的路径
    let import_path = extract_quoted_string(path_part)?;

    // 从路径中提取模块名
    let module_name = extract_vue3_module_name(&import_path, import_prefix)?;

    Some((identifier, module_name))
}

/// 解析顶层懒加载常量声明，返回 (标识符, 模块名)
///
/// 匹配模式：`const XxxView = () => import('@/views/xxx/...')`
fn parse_lazy_const_import(line: &str, import_prefix: &str) -> Option<(String, String)> {
    if !line.starts_with("const ") {
        return None;
    }

    // 必须包含 "import(" 关键字
    if !line.contains("import(") {
        return None;
    }

    let after_const = line.strip_prefix("const ")?.trim_start();
    let eq_pos = after_const.find('=')?;
    let identifier = after_const[..eq_pos].trim().to_string();

    // 提取 import('...') 中的路径
    let import_path = extract_import_call_path(line)?;
    let module_name = extract_vue3_module_name(&import_path, import_prefix)?;

    Some((identifier, module_name))
}

/// 从引号包裹的字符串中提取内容（支持单引号和双引号）
fn extract_quoted_string(s: &str) -> Option<String> {
    let s = s.trim().trim_end_matches(';');
    if (s.starts_with('\'') && s.ends_with('\'')) || (s.starts_with('"') && s.ends_with('"')) {
        Some(s[1..s.len() - 1].to_string())
    } else {
        None
    }
}

/// 从 `import('...')` 调用中提取路径
fn extract_import_call_path(line: &str) -> Option<String> {
    let start = line.find("import(")? + "import(".len();
    let rest = &line[start..];
    let end = rest.find(')')?;
    let inner = rest[..end].trim();
    extract_quoted_string(inner)
}

/// 判断一行是否是路由对象的开始
///
/// 路由对象通常以 `{` 开头（可能前面有空格或逗号），
/// 且包含 path/component/name 等路由属性的上下文中
fn is_route_object_start(trimmed: &str) -> bool {
    // 必须以 { 开头
    if !trimmed.starts_with('{') {
        return false;
    }
    // 排除解构赋值（如 `const { createRouter } = ...`）和非路由对象
    // 路由对象通常包含 path/component/name 等关键字
    // 简单启发式：如果同一行包含路由特征关键字，或者是纯 { 开头（多行路由对象），则认为是路由对象
    let rest = &trimmed[1..].trim_start();
    // 纯 `{` 或 `{` 后跟路由特征关键字（path:, name:, component:, redirect:, children:）
    if rest.is_empty() || *rest == "}" {
        return true;
    }
    // 检查是否包含路由对象的典型属性
    rest.starts_with("path:")
        || rest.starts_with("path :")
        || rest.starts_with("name:")
        || rest.starts_with("name :")
        || rest.starts_with("component:")
        || rest.starts_with("component :")
        || rest.starts_with("redirect:")
        || rest.starts_with("redirect :")
        || rest.starts_with("children:")
        || rest.starts_with("children :")
        || rest.starts_with("meta:")
        || rest.starts_with("meta :")
}

/// 从指定行开始，收集完整的花括号块（处理嵌套）
///
/// 返回 (块内所有行, 结束行索引)
fn collect_brace_block(lines: &[&str], start: usize) -> (Vec<String>, usize) {
    let mut depth = 0i32;
    let mut block = Vec::new();
    let mut end = start;

    for (idx, &line) in lines.iter().enumerate().skip(start) {
        block.push(line.to_string());
        for ch in line.chars() {
            match ch {
                '{' => depth += 1,
                '}' => depth -= 1,
                _ => {}
            }
        }
        end = idx;
        if depth <= 0 {
            break;
        }
    }

    (block, end)
}

/// 判断路由对象块是否应被移除
///
/// 移除条件（满足任一）：
/// 1. component 属性引用了已被移除的静态 import 标识符
/// 2. 包含指向未选中模块的动态 import() 调用
fn should_remove_route_block(
    block_text: &str,
    selected: &HashSet<&str>,
    removed_identifiers: &HashSet<String>,
    import_prefix: &str,
) -> bool {
    for line in block_text.lines() {
        let trimmed = line.trim();

        // 检查 component: XxxView（静态引用）
        if trimmed.starts_with("component:") || trimmed.starts_with("component :") {
            let after_component = trimmed
                .strip_prefix("component:")
                .or_else(|| trimmed.strip_prefix("component :"))
                .unwrap_or("")
                .trim()
                .trim_end_matches(',');

            // 如果引用了被移除的标识符 → 移除此路由
            if removed_identifiers.contains(after_component) {
                return true;
            }
        }

        // 检查动态 import()：component: () => import('@/views/xxx/...')
        if trimmed.contains("import(") {
            if let Some(import_path) = extract_import_call_path(trimmed) {
                if let Some(module_name) =
                    extract_vue3_module_name(&import_path, import_prefix)
                {
                    if !selected.contains(module_name.as_str()) {
                        return true;
                    }
                }
            }
        }
    }

    false
}

// ============================================================================
// Python 导入重写核心逻辑（供 FastApiImportRewriter 使用）
// ============================================================================

/// 重写 Python 文件中的模块导入，只保留选中模块相关的行
fn rewrite_python_imports(
    content: &str,
    selected_modules: &[String],
    modules_dir: &str,
) -> String {
    let selected: HashSet<&str> = selected_modules.iter().map(|s| s.as_str()).collect();

    // 将 modules_dir 中的 "/" 替换为 "."，适配 Python import 语法
    // 例如 "src/views" → "src.views"
    let import_prefix = modules_dir.replace('/', ".");

    // 第一遍：扫描所有 import 行，建立 "别名 → 模块名" 映射
    let mut alias_map: HashMap<String, String> = HashMap::new();
    for line in content.lines() {
        collect_aliases(line.trim(), &import_prefix, &mut alias_map);
    }

    // 第二遍：逐行过滤
    let mut output: Vec<String> = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();

        // 情况 1: from {prefix}.xxx... import ...
        if let Some(module_name) = extract_module_from_from_import(trimmed, &import_prefix) {
            if selected.contains(module_name.as_str()) {
                output.push(line.to_string());
            }
            continue;
        }

        // 情况 2: from {prefix} import xxx, yyy
        if let Some(names) = extract_names_from_bulk_import(trimmed, &import_prefix) {
            let kept: Vec<&str> = names
                .iter()
                .filter(|n| selected.contains(n.as_str()))
                .map(|s| s.as_str())
                .collect();
            if kept.is_empty() {
                continue; // 全部未选中 → 移除此行
            }
            if kept.len() == names.len() {
                output.push(line.to_string()); // 全部保留 → 原样
            } else {
                // 部分保留 → 重写
                output.push(format!("from {} import {}", import_prefix, kept.join(", ")));
            }
            continue;
        }

        // 情况 3: app.include_router(...) 行
        if trimmed.contains("include_router(") {
            if should_remove_router_line(trimmed, &selected, &alias_map, &import_prefix) {
                continue; // 未选中模块的 router → 移除
            }
        }

        // 其他行 → 原样保留
        output.push(line.to_string());
    }

    output.join("\n")
}

// ============================================================================
// 解析辅助函数
// ============================================================================

/// 从 `from {prefix}.xxx...` 格式的 import 行中提取顶层模块名
///
/// 例如：
/// - `from modules.auth.routes import router` → Some("auth")
/// - `from modules.users import models` → Some("users")
/// - `from fastapi import FastAPI` → None
fn extract_module_from_from_import(line: &str, prefix: &str) -> Option<String> {
    if !line.starts_with("from ") {
        return None;
    }

    let after_from = line.strip_prefix("from ")?.trim_start();
    let import_pos = after_from.find(" import ")?;
    let module_path = after_from[..import_pos].trim();

    // 检查是否以 prefix. 开头
    let after_prefix = module_path.strip_prefix(prefix)?.strip_prefix('.')?;

    // 取第一个 "." 之前的部分作为模块名
    let module_name = match after_prefix.find('.') {
        Some(pos) => &after_prefix[..pos],
        None => after_prefix,
    };

    if module_name.is_empty() {
        return None;
    }

    Some(module_name.to_string())
}

/// 从 `from {prefix} import xxx, yyy` 格式中提取模块名列表
fn extract_names_from_bulk_import(line: &str, prefix: &str) -> Option<Vec<String>> {
    let expected_start = format!("from {} import ", prefix);
    if !line.starts_with(&expected_start) {
        return None;
    }

    let names_part = line.strip_prefix(&expected_start)?;
    let names: Vec<String> = names_part
        .split(',')
        .map(|s| {
            let s = s.trim();
            // 处理 "xxx as yyy" 的情况，取原始名
            match s.find(" as ") {
                Some(pos) => s[..pos].trim().to_string(),
                None => s.to_string(),
            }
        })
        .filter(|s| !s.is_empty())
        .collect();

    if names.is_empty() {
        return None;
    }

    Some(names)
}

/// 收集 import 行中的别名映射（"别名 → 模块名"）
fn collect_aliases(line: &str, prefix: &str, alias_map: &mut HashMap<String, String>) {
    // 情况 1: from {prefix}.xxx... import yyy as zzz
    if let Some(module_name) = extract_module_from_from_import(line, prefix) {
        if let Some(import_pos) = line.find(" import ") {
            let imports_part = &line[import_pos + 8..];
            for item in imports_part.split(',') {
                let item = item.trim();
                if let Some(as_pos) = item.find(" as ") {
                    let alias = item[as_pos + 4..].trim();
                    alias_map.insert(alias.to_string(), module_name.clone());
                }
            }
        }
        // 始终记录模块名自身
        alias_map.insert(module_name.clone(), module_name);
    }

    // 情况 2: from {prefix} import xxx, yyy
    if let Some(names) = extract_names_from_bulk_import(line, prefix) {
        for name in &names {
            alias_map.insert(name.clone(), name.clone());
        }
        // 处理 as 别名
        if let Some(import_pos) = line.find(" import ") {
            let imports_part = &line[import_pos + 8..];
            for item in imports_part.split(',') {
                let item = item.trim();
                if let Some(as_pos) = item.find(" as ") {
                    let original = item[..as_pos].trim();
                    let alias = item[as_pos + 4..].trim();
                    alias_map.insert(alias.to_string(), original.to_string());
                }
            }
        }
    }
}

/// 判断 include_router 行是否应该被移除
fn should_remove_router_line(
    line: &str,
    selected: &HashSet<&str>,
    alias_map: &HashMap<String, String>,
    prefix: &str,
) -> bool {
    let ref_name = match extract_router_ref(line) {
        Some(name) => name,
        None => return false, // 无法解析 → 保留（安全策略）
    };

    // 策略 1：直接在别名映射中查找
    if let Some(module_name) = alias_map.get(&ref_name) {
        return !selected.contains(module_name.as_str());
    }

    // 策略 2：xxx_router / xxx_routes 命名约定
    let base = ref_name
        .trim_end_matches("_router")
        .trim_end_matches("_routes");
    if base != ref_name {
        if let Some(module_name) = alias_map.get(base) {
            return !selected.contains(module_name.as_str());
        }
    }

    // 策略 3：点号引用（auth.router / modules.auth.router）
    if ref_name.contains('.') {
        // 尝试 prefix.xxx.router 模式
        let dotted_prefix = format!("{}.", prefix);
        if let Some(rest) = ref_name.strip_prefix(&dotted_prefix) {
            let module_name = match rest.find('.') {
                Some(pos) => &rest[..pos],
                None => rest,
            };
            if alias_map.contains_key(module_name) {
                return !selected.contains(module_name);
            }
        }

        // 尝试 xxx.router 模式
        if let Some(dot_pos) = ref_name.find('.') {
            let module_ref = &ref_name[..dot_pos];
            if let Some(module_name) = alias_map.get(module_ref) {
                return !selected.contains(module_name.as_str());
            }
        }
    }

    // 无法关联到任何模块 → 保留
    false
}

/// 从 include_router(...) 调用中提取第一个参数
fn extract_router_ref(line: &str) -> Option<String> {
    let start = line.find("include_router(")? + "include_router(".len();
    let rest = &line[start..];
    let end = rest
        .find(|c: char| c == ',' || c == ')')
        .unwrap_or(rest.len());
    let ref_name = rest[..end].trim();

    if ref_name.is_empty() {
        return None;
    }

    Some(ref_name.to_string())
}

// ============================================================================
// 导入完整性校验函数
// ============================================================================

/// 校验 Python 入口文件中所有 `from {modules_dir}.xxx` 导入引用的模块目录是否存在
///
/// 扫描重写后的 main.py，提取所有 `from modules.xxx...` 行中的模块名，
/// 检查 `build_dir/{modules_dir}/{module_name}/` 是否存在。
fn validate_python_imports(content: &str, build_dir: &Path, modules_dir: &str) -> Vec<String> {
    let import_prefix = modules_dir.replace('/', ".");
    let mut missing: Vec<String> = Vec::new();
    let mut checked: HashSet<String> = HashSet::new();

    for line in content.lines() {
        let trimmed = line.trim();

        // 情况 1: from {prefix}.xxx... import ...
        if let Some(module_name) = extract_module_from_from_import(trimmed, &import_prefix) {
            if checked.insert(module_name.clone()) {
                let module_path = build_dir.join(modules_dir).join(&module_name);
                if !module_path.exists() {
                    missing.push(format!("{}/{}", modules_dir, module_name));
                }
            }
            continue;
        }

        // 情况 2: from {prefix} import xxx, yyy
        if let Some(names) = extract_names_from_bulk_import(trimmed, &import_prefix) {
            for name in names {
                if checked.insert(name.clone()) {
                    let module_path = build_dir.join(modules_dir).join(&name);
                    if !module_path.exists() {
                        missing.push(format!("{}/{}", modules_dir, name));
                    }
                }
            }
        }
    }

    missing
}

/// 校验 Vue3 router 入口文件中所有模块导入引用的目录是否存在
///
/// 扫描重写后的 router/index.ts，提取所有 `import ... from '@/views/xxx/...'`
/// 和 `import('@/views/xxx/...')` 中的模块名，
/// 检查 `build_dir/{modules_dir}/{module_name}/` 是否存在。
fn validate_vue3_imports(content: &str, build_dir: &Path, modules_dir: &str) -> Vec<String> {
    let import_prefix = to_vue3_import_prefix(modules_dir);
    let mut missing: Vec<String> = Vec::new();
    let mut checked: HashSet<String> = HashSet::new();

    for line in content.lines() {
        let trimmed = line.trim();

        // 静态 import: import XxxView from '@/views/xxx/...'
        if let Some((_ident, module_name)) = parse_static_import(trimmed, &import_prefix) {
            if checked.insert(module_name.clone()) {
                let module_path = build_dir.join(modules_dir).join(&module_name);
                if !module_path.exists() {
                    missing.push(format!("{}/{}", modules_dir, module_name));
                }
            }
            continue;
        }

        // 顶层懒加载: const XxxView = () => import('@/views/xxx/...')
        if let Some((_ident, module_name)) = parse_lazy_const_import(trimmed, &import_prefix) {
            if checked.insert(module_name.clone()) {
                let module_path = build_dir.join(modules_dir).join(&module_name);
                if !module_path.exists() {
                    missing.push(format!("{}/{}", modules_dir, module_name));
                }
            }
            continue;
        }

        // 内联动态 import: component: () => import('@/views/xxx/...')
        if let Some(import_path) = extract_import_call_path(trimmed) {
            if let Some(module_name) = extract_vue3_module_name(&import_path, &import_prefix) {
                if checked.insert(module_name.clone()) {
                    let module_path = build_dir.join(modules_dir).join(&module_name);
                    if !module_path.exists() {
                        missing.push(format!("{}/{}", modules_dir, module_name));
                    }
                }
            }
        }
    }

    missing
}


// ============================================================================
// 单元测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // -----------------------------------------------------------------------
    // 测试 3 种 import 模式的过滤
    // -----------------------------------------------------------------------

    #[test]
    fn test_from_module_import_filtering() {
        // 模式 1: from modules.xxx.routes import router as xxx_router
        let content = "\
from fastapi import FastAPI
from modules.auth.routes import router as auth_router
from modules.users.routes import router as users_router
from modules.orders.routes import router as orders_router

app = FastAPI()
app.include_router(auth_router)
app.include_router(users_router)
app.include_router(orders_router)";

        let selected = vec!["auth".to_string(), "orders".to_string()];
        let result = rewrite_python_imports(content, &selected, "modules");

        assert!(result.contains("from modules.auth.routes import router as auth_router"));
        assert!(!result.contains("users"));
        assert!(result.contains("from modules.orders.routes import router as orders_router"));
        assert!(result.contains("app.include_router(auth_router)"));
        assert!(!result.contains("app.include_router(users_router)"));
        assert!(result.contains("app.include_router(orders_router)"));
    }

    #[test]
    fn test_from_module_import_submodule() {
        // 模式 2: from modules.xxx import routes as xxx_routes
        let content = "\
from modules.auth import routes as auth_routes
from modules.users import routes as users_routes

app.include_router(auth_routes.router)
app.include_router(users_routes.router)";

        let selected = vec!["auth".to_string()];
        let result = rewrite_python_imports(content, &selected, "modules");

        assert!(result.contains("from modules.auth import routes as auth_routes"));
        assert!(!result.contains("users"));
    }

    #[test]
    fn test_bulk_import_filtering() {
        // 模式 3: from modules import xxx, yyy
        let content = "\
from modules import auth, users, orders

app.include_router(auth.router)
app.include_router(users.router)
app.include_router(orders.router)";

        let selected = vec!["auth".to_string(), "orders".to_string()];
        let result = rewrite_python_imports(content, &selected, "modules");

        assert!(result.contains("from modules import auth, orders"));
        assert!(!result.contains("users"));
        assert!(result.contains("app.include_router(auth.router)"));
        assert!(result.contains("app.include_router(orders.router)"));
    }

    // -----------------------------------------------------------------------
    // 边界情况
    // -----------------------------------------------------------------------

    #[test]
    fn test_non_module_lines_preserved() {
        // 非模块相关的行应原样保留
        let content = "\
from fastapi import FastAPI
import uvicorn

app = FastAPI()

if __name__ == '__main__':
    uvicorn.run(app)";

        let selected = vec!["auth".to_string()];
        let result = rewrite_python_imports(content, &selected, "modules");

        assert_eq!(result, content);
    }

    #[test]
    fn test_empty_content() {
        let result = rewrite_python_imports("", &[], "modules");
        assert_eq!(result, "");
    }

    #[test]
    fn test_custom_modules_dir() {
        // 自定义模块目录名
        let content = "\
from plugins.auth.routes import router as auth_router
from plugins.users.routes import router as users_router";

        let selected = vec!["auth".to_string()];
        let result = rewrite_python_imports(content, &selected, "plugins");

        assert!(result.contains("from plugins.auth.routes import router as auth_router"));
        assert!(!result.contains("users"));
    }

    #[test]
    fn test_dotted_router_ref() {
        // 点号引用：modules.auth.router
        let content = "\
from modules import auth, users

app.include_router(modules.auth.router)
app.include_router(modules.users.router)";

        let selected = vec!["auth".to_string()];
        let result = rewrite_python_imports(content, &selected, "modules");

        assert!(result.contains("app.include_router(modules.auth.router)"));
        assert!(!result.contains("modules.users.router"));
    }

    // -----------------------------------------------------------------------
    // process_entry_file 集成测试
    // -----------------------------------------------------------------------

    #[test]
    fn test_process_entry_file_missing_file() {
        // 入口文件不存在时应跳过，不报错
        let tmp = TempDir::new().unwrap();
        let rewriter = FastApiImportRewriter;
        let result = process_entry_file(&rewriter, tmp.path(), &[], "modules");
        assert!(result.is_ok());
    }

    #[test]
    fn test_process_entry_file_normal_rewrite() {
        // 正常重写流程
        let tmp = TempDir::new().unwrap();
        let main_py = tmp.path().join("main.py");
        std::fs::write(
            &main_py,
            "from modules.auth.routes import router as auth_router\n\
             from modules.users.routes import router as users_router\n\
             app.include_router(auth_router)\n\
             app.include_router(users_router)\n",
        )
        .unwrap();

        let rewriter = FastApiImportRewriter;
        let selected = vec!["auth".to_string()];
        process_entry_file(&rewriter, tmp.path(), &selected, "modules").unwrap();

        let result = std::fs::read_to_string(&main_py).unwrap();
        assert!(result.contains("auth_router"));
        assert!(!result.contains("users_router"));
    }

    // -----------------------------------------------------------------------
    // Vue3 ImportRewriter 测试
    // -----------------------------------------------------------------------

    #[test]
    fn test_vue3_static_import_filtering() {
        // 模式 1：静态 import + component 引用
        let content = "\
import { createRouter, createWebHistory } from 'vue-router'
import DashboardView from '@/views/dashboard/index.vue'
import LoginView from '@/views/login/index.vue'
import SettingsView from '@/views/settings/index.vue'

const routes = [
  {
    path: '/dashboard',
    component: DashboardView,
  },
  {
    path: '/login',
    component: LoginView,
  },
  {
    path: '/settings',
    component: SettingsView,
  },
]

export default createRouter({
  history: createWebHistory(),
  routes,
})";

        let selected = vec!["dashboard".to_string(), "settings".to_string()];
        let result = rewrite_vue3_router(content, &selected, "src/views");

        // 保留 dashboard 和 settings 的 import
        assert!(result.contains("import DashboardView from '@/views/dashboard/index.vue'"));
        assert!(result.contains("import SettingsView from '@/views/settings/index.vue'"));
        // 移除 login 的 import
        assert!(!result.contains("LoginView"));
        // 保留 vue-router 的 import（非模块 import）
        assert!(result.contains("import { createRouter, createWebHistory } from 'vue-router'"));
        // 保留 dashboard 和 settings 的路由对象
        assert!(result.contains("'/dashboard'"));
        assert!(result.contains("'/settings'"));
        // 移除 login 的路由对象
        assert!(!result.contains("'/login'"));
    }

    #[test]
    fn test_vue3_dynamic_import_filtering() {
        // 模式 2：动态懒加载 import()
        let content = "\
import { createRouter, createWebHistory } from 'vue-router'

const routes = [
  {
    path: '/dashboard',
    component: () => import('@/views/dashboard/index.vue'),
  },
  {
    path: '/login',
    component: () => import('@/views/login/index.vue'),
  },
  {
    path: '/settings',
    component: () => import('@/views/settings/index.vue'),
  },
]

export default createRouter({
  history: createWebHistory(),
  routes,
})";

        let selected = vec!["dashboard".to_string()];
        let result = rewrite_vue3_router(content, &selected, "src/views");

        // 保留 dashboard 路由
        assert!(result.contains("'/dashboard'"));
        assert!(result.contains("@/views/dashboard/index.vue"));
        // 移除 login 和 settings 路由
        assert!(!result.contains("'/login'"));
        assert!(!result.contains("'/settings'"));
        // 保留 vue-router import 和 createRouter
        assert!(result.contains("createRouter"));
    }

    #[test]
    fn test_vue3_const_lazy_import_filtering() {
        // 模式 2 变体：const Xxx = () => import('...')
        let content = "\
import { createRouter, createWebHistory } from 'vue-router'

const DashboardView = () => import('@/views/dashboard/index.vue')
const LoginView = () => import('@/views/login/index.vue')

const routes = [
  {
    path: '/dashboard',
    component: DashboardView,
  },
  {
    path: '/login',
    component: LoginView,
  },
]

export default createRouter({
  history: createWebHistory(),
  routes,
})";

        let selected = vec!["dashboard".to_string()];
        let result = rewrite_vue3_router(content, &selected, "src/views");

        // 保留 dashboard
        assert!(result.contains("const DashboardView"));
        assert!(result.contains("'/dashboard'"));
        // 移除 login
        assert!(!result.contains("LoginView"));
        assert!(!result.contains("'/login'"));
    }

    #[test]
    fn test_vue3_mixed_import_styles() {
        // 混合模式：部分静态 import，部分动态 import
        let content = "\
import { createRouter, createWebHistory } from 'vue-router'
import DashboardView from '@/views/dashboard/index.vue'

const routes = [
  {
    path: '/dashboard',
    component: DashboardView,
  },
  {
    path: '/login',
    component: () => import('@/views/login/index.vue'),
  },
  {
    path: '/settings',
    component: () => import('@/views/settings/index.vue'),
  },
]";

        let selected = vec!["dashboard".to_string(), "login".to_string()];
        let result = rewrite_vue3_router(content, &selected, "src/views");

        assert!(result.contains("DashboardView"));
        assert!(result.contains("'/dashboard'"));
        assert!(result.contains("'/login'"));
        assert!(!result.contains("'/settings'"));
    }

    #[test]
    fn test_vue3_custom_modules_dir() {
        // 自定义模块目录：src/pages 而非 src/views
        let content = "\
import HomeView from '@/pages/home/index.vue'
import AboutView from '@/pages/about/index.vue'

const routes = [
  {
    path: '/',
    component: HomeView,
  },
  {
    path: '/about',
    component: AboutView,
  },
]";

        let selected = vec!["home".to_string()];
        let result = rewrite_vue3_router(content, &selected, "src/pages");

        assert!(result.contains("HomeView"));
        assert!(result.contains("'/'"));
        assert!(!result.contains("AboutView"));
        assert!(!result.contains("'/about'"));
    }

    #[test]
    fn test_vue3_non_module_imports_preserved() {
        // 非模块相关的 import 应原样保留
        let content = "\
import { createRouter, createWebHistory } from 'vue-router'
import type { RouteRecordRaw } from 'vue-router'
import { useAuth } from '@/composables/useAuth'

const routes: RouteRecordRaw[] = []

export default createRouter({
  history: createWebHistory(),
  routes,
})";

        let selected: Vec<String> = vec![];
        let result = rewrite_vue3_router(content, &selected, "src/views");

        // 所有非模块 import 应保留
        assert!(result.contains("import { createRouter, createWebHistory } from 'vue-router'"));
        assert!(result.contains("import type { RouteRecordRaw } from 'vue-router'"));
        assert!(result.contains("import { useAuth } from '@/composables/useAuth'"));
    }

    #[test]
    fn test_vue3_empty_content() {
        let result = rewrite_vue3_router("", &[], "src/views");
        assert_eq!(result, "");
    }

    #[test]
    fn test_vue3_nested_module_path() {
        // 嵌套路径：@/views/system/user/index.vue → 模块名应为 "system"
        let content = "\
import UserView from '@/views/system/user/index.vue'
import RoleView from '@/views/system/role/index.vue'
import DashboardView from '@/views/dashboard/index.vue'

const routes = [
  {
    path: '/system/user',
    component: UserView,
  },
  {
    path: '/system/role',
    component: RoleView,
  },
  {
    path: '/dashboard',
    component: DashboardView,
  },
]";

        // 选中 "system" 模块 → 保留 system 下的所有子路由
        let selected = vec!["system".to_string()];
        let result = rewrite_vue3_router(content, &selected, "src/views");

        assert!(result.contains("UserView"));
        assert!(result.contains("RoleView"));
        assert!(!result.contains("DashboardView"));
    }

    #[test]
    fn test_vue3_get_rewriter_returns_some() {
        // get_rewriter("vue3") 应返回 Some
        let rewriter = get_rewriter("vue3");
        assert!(rewriter.is_some());
        assert_eq!(rewriter.unwrap().entry_file(), "src/router/index.ts");
    }

    #[test]
    fn test_vue3_process_entry_file_integration() {
        // Vue3 入口文件重写集成测试
        let tmp = TempDir::new().unwrap();
        let router_dir = tmp.path().join("src").join("router");
        std::fs::create_dir_all(&router_dir).unwrap();
        let router_file = router_dir.join("index.ts");
        std::fs::write(
            &router_file,
            "import DashboardView from '@/views/dashboard/index.vue'\n\
             import LoginView from '@/views/login/index.vue'\n\
             \n\
             const routes = [\n\
               {\n\
                 path: '/dashboard',\n\
                 component: DashboardView,\n\
               },\n\
               {\n\
                 path: '/login',\n\
                 component: LoginView,\n\
               },\n\
             ]\n",
        )
        .unwrap();

        let rewriter = Vue3ImportRewriter;
        let selected = vec!["dashboard".to_string()];
        process_entry_file(&rewriter, tmp.path(), &selected, "src/views").unwrap();

        let result = std::fs::read_to_string(&router_file).unwrap();
        assert!(result.contains("DashboardView"));
        assert!(!result.contains("LoginView"));
        assert!(result.contains("'/dashboard'"));
        assert!(!result.contains("'/login'"));
    }

    // ================================================================
    // 导入完整性校验测试
    // ================================================================

    #[test]
    fn test_validate_python_imports_all_exist() {
        // 所有导入的模块目录都存在 → 校验通过
        let tmp = TempDir::new().unwrap();
        std::fs::create_dir_all(tmp.path().join("modules/auth")).unwrap();
        std::fs::create_dir_all(tmp.path().join("modules/users")).unwrap();

        let content = "from modules.auth.routes import router as auth_router\n\
                        from modules.users import models\n";

        let missing = validate_python_imports(content, tmp.path(), "modules");
        assert!(missing.is_empty(), "应该没有缺失: {:?}", missing);
    }

    #[test]
    fn test_validate_python_imports_missing_module() {
        // 引用了不存在的模块 → 返回缺失列表
        let tmp = TempDir::new().unwrap();
        std::fs::create_dir_all(tmp.path().join("modules/auth")).unwrap();
        // 注意：没有创建 modules/users

        let content = "from modules.auth.routes import router\n\
                        from modules.users import models\n";

        let missing = validate_python_imports(content, tmp.path(), "modules");
        assert_eq!(missing.len(), 1);
        assert_eq!(missing[0], "modules/users");
    }

    #[test]
    fn test_validate_python_bulk_import_missing() {
        // from modules import xxx, yyy 格式，部分模块不存在
        let tmp = TempDir::new().unwrap();
        std::fs::create_dir_all(tmp.path().join("modules/auth")).unwrap();

        let content = "from modules import auth, billing\n";

        let missing = validate_python_imports(content, tmp.path(), "modules");
        assert_eq!(missing.len(), 1);
        assert_eq!(missing[0], "modules/billing");
    }

    #[test]
    fn test_validate_python_no_module_imports() {
        // 没有模块导入行 → 校验通过
        let tmp = TempDir::new().unwrap();
        let content = "from fastapi import FastAPI\nimport uvicorn\n";

        let missing = validate_python_imports(content, tmp.path(), "modules");
        assert!(missing.is_empty());
    }

    #[test]
    fn test_validate_vue3_imports_all_exist() {
        // 所有导入的 views 目录都存在 → 校验通过
        let tmp = TempDir::new().unwrap();
        std::fs::create_dir_all(tmp.path().join("src/views/dashboard")).unwrap();
        std::fs::create_dir_all(tmp.path().join("src/views/login")).unwrap();

        let content = "import DashboardView from '@/views/dashboard/index.vue'\n\
                        import LoginView from '@/views/login/index.vue'\n";

        let missing = validate_vue3_imports(content, tmp.path(), "src/views");
        assert!(missing.is_empty(), "应该没有缺失: {:?}", missing);
    }

    #[test]
    fn test_validate_vue3_imports_missing_module() {
        // 引用了不存在的 views 目录 → 返回缺失列表
        let tmp = TempDir::new().unwrap();
        std::fs::create_dir_all(tmp.path().join("src/views/dashboard")).unwrap();

        let content = "import DashboardView from '@/views/dashboard/index.vue'\n\
                        import SettingsView from '@/views/settings/index.vue'\n";

        let missing = validate_vue3_imports(content, tmp.path(), "src/views");
        assert_eq!(missing.len(), 1);
        assert_eq!(missing[0], "src/views/settings");
    }

    #[test]
    fn test_validate_vue3_dynamic_import_missing() {
        // 动态 import() 引用不存在的模块
        let tmp = TempDir::new().unwrap();
        std::fs::create_dir_all(tmp.path().join("src/views/dashboard")).unwrap();

        let content = "const DashboardView = () => import('@/views/dashboard/index.vue')\n\
                        const AdminView = () => import('@/views/admin/index.vue')\n";

        let missing = validate_vue3_imports(content, tmp.path(), "src/views");
        assert_eq!(missing.len(), 1);
        assert_eq!(missing[0], "src/views/admin");
    }

    #[test]
    fn test_validate_vue3_no_module_imports() {
        // 没有 views 相关导入 → 校验通过
        let tmp = TempDir::new().unwrap();
        let content = "import { createRouter } from 'vue-router'\n";

        let missing = validate_vue3_imports(content, tmp.path(), "src/views");
        assert!(missing.is_empty());
    }

    #[test]
    fn test_validate_entry_file_missing_file_skips() {
        // 入口文件不存在时跳过校验（不报错）
        let tmp = TempDir::new().unwrap();
        let rewriter = FastApiImportRewriter;
        let result = validate_entry_file(&rewriter, tmp.path(), "modules");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_entry_file_returns_error_on_missing_module() {
        // 入口文件存在但引用了不存在的模块 → 返回错误
        let tmp = TempDir::new().unwrap();
        std::fs::create_dir_all(tmp.path().join("modules/auth")).unwrap();
        std::fs::write(
            tmp.path().join("main.py"),
            "from modules.auth.routes import router\nfrom modules.ghost import api\n",
        )
        .unwrap();

        let rewriter = FastApiImportRewriter;
        let result = validate_entry_file(&rewriter, tmp.path(), "modules");
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("modules/ghost"), "错误信息应包含缺失模块: {}", err_msg);
    }
}
