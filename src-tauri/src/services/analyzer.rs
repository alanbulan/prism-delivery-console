// ============================================================================
// 文件分析服务：递归遍历项目文件、哈希计算、依赖推断
// ✅ 只能做：文件遍历、SHA256 哈希计算、import 语句解析
// ⛔ 禁止：依赖 tauri::*，直接操作数据库
// ============================================================================

use regex::Regex;
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::path::Path;
use walkdir::WalkDir;

/// 文件索引条目（单个文件的元信息）
#[derive(Debug, Clone)]
pub struct FileEntry {
    /// 相对于项目根目录的文件路径
    pub relative_path: String,
    /// 文件内容的 SHA256 哈希值（十六进制）
    pub file_hash: String,
}

/// 扫描时需要忽略的目录名
const IGNORED_DIRS: &[&str] = &[
    ".git",
    "node_modules",
    "__pycache__",
    ".venv",
    "venv",
    ".idea",
    ".vscode",
    "target",
    "dist",
    "build",
    ".next",
    ".nuxt",
];

/// 递归遍历项目目录，计算每个文件的 SHA256 哈希
///
/// # 参数
/// - `project_path`: 项目根目录路径
///
/// # 返回
/// - `Ok(Vec<FileEntry>)`: 所有文件的索引条目
/// - `Err(String)`: 遍历失败的错误描述
pub fn scan_project_files(project_path: &Path) -> Result<Vec<FileEntry>, String> {
    if !project_path.exists() {
        return Err(format!("项目路径不存在：{}", project_path.display()));
    }

    let mut entries = Vec::new();

    for entry in WalkDir::new(project_path)
        .into_iter()
        .filter_entry(|e| {
            // 过滤掉忽略目录
            if e.file_type().is_dir() {
                if let Some(name) = e.file_name().to_str() {
                    return !IGNORED_DIRS.contains(&name);
                }
            }
            true
        })
    {
        let entry = entry.map_err(|e| format!("遍历文件失败：{}", e))?;

        // 只处理文件，跳过目录
        if !entry.file_type().is_file() {
            continue;
        }

        let abs_path = entry.path();

        // 计算相对路径
        let relative = abs_path
            .strip_prefix(project_path)
            .map_err(|e| format!("计算相对路径失败：{}", e))?
            .to_string_lossy()
            .replace('\\', "/"); // 统一使用正斜杠

        // 计算文件 SHA256 哈希
        let hash = compute_file_hash(abs_path)?;

        entries.push(FileEntry {
            relative_path: relative,
            file_hash: hash,
        });
    }

    Ok(entries)
}

/// 计算单个文件的 SHA256 哈希值
fn compute_file_hash(path: &Path) -> Result<String, String> {
    let content = std::fs::read(path)
        .map_err(|e| format!("读取文件失败 {}: {}", path.display(), e))?;
    let mut hasher = Sha256::new();
    hasher.update(&content);
    let result = hasher.finalize();
    Ok(format!("{:x}", result))
}

// ============================================================================
// 依赖推断
// ============================================================================

/// 依赖边（源文件 → 目标文件）
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DependencyEdge {
    /// 源文件相对路径
    pub source: String,
    /// 目标文件相对路径
    pub target: String,
}

/// 从项目文件中提取 import 依赖关系
///
/// 支持的语法：
/// - Python: `from xxx import ...` / `import xxx`
/// - JS/TS: `import ... from '...'` / `require('...')`
///
/// 仅保留项目内部的相对引用（以 `.` 或 `..` 开头），忽略第三方包
///
/// # 参数
/// - `project_path`: 项目根目录
/// - `file_paths`: 已知的项目文件相对路径列表
///
/// # 返回
/// - 依赖边列表
pub fn extract_dependencies(
    project_path: &Path,
    file_paths: &[String],
) -> Result<Vec<DependencyEdge>, String> {
    // 构建已知文件集合，用于验证目标是否存在
    let known_files: HashSet<&str> = file_paths.iter().map(|s| s.as_str()).collect();

    // JS/TS import 正则：匹配 import ... from '...' 和 require('...')
    let re_js_import = Regex::new(
        r#"(?:import\s+.*?\s+from\s+['"]([^'"]+)['"]|require\s*\(\s*['"]([^'"]+)['"]\s*\))"#,
    )
    .map_err(|e| format!("正则编译失败：{}", e))?;

    // Python from import 正则：匹配 from xxx import ...（相对和绝对）
    let re_py_from = Regex::new(r#"^from\s+(\.{0,3}\w[\w.]*|\.+)\s+import"#)
        .map_err(|e| format!("正则编译失败：{}", e))?;

    // Python import 正则：匹配 import xxx（绝对导入）
    let re_py_import = Regex::new(r#"^import\s+([\w][\w.]*)"#)
        .map_err(|e| format!("正则编译失败：{}", e))?;

    let mut edges = Vec::new();

    for source_path in file_paths {
        let abs_path = project_path.join(source_path);

        // 只处理代码文件
        if !is_code_file(source_path) {
            continue;
        }

        // 读取文件内容（忽略读取失败的文件）
        let content = match std::fs::read_to_string(&abs_path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        // 获取源文件所在目录（相对路径）
        let source_dir = Path::new(source_path)
            .parent()
            .map(|p| p.to_string_lossy().replace('\\', "/"))
            .unwrap_or_default();

        for line in content.lines() {
            let trimmed = line.trim();

            // 跳过注释行
            if trimmed.starts_with('#') || trimmed.starts_with("//") {
                continue;
            }

            // JS/TS import 解析
            if let Some(caps) = re_js_import.captures(trimmed) {
                let raw_path = caps
                    .get(1)
                    .or_else(|| caps.get(2))
                    .map(|m| m.as_str())
                    .unwrap_or("");

                // 只处理相对路径引用
                if raw_path.starts_with('.') {
                    if let Some(target) =
                        resolve_js_import(&source_dir, raw_path, &known_files)
                    {
                        edges.push(DependencyEdge {
                            source: source_path.clone(),
                            target,
                        });
                    }
                }
            }

            // Python from import 解析（相对 + 绝对）
            if let Some(caps) = re_py_from.captures(trimmed) {
                let module_path = &caps[1];
                if module_path.starts_with('.') {
                    // 相对导入：from .xxx import / from ..xxx import
                    if let Some(target) =
                        resolve_py_import(&source_dir, module_path, &known_files)
                    {
                        edges.push(DependencyEdge {
                            source: source_path.clone(),
                            target,
                        });
                    }
                } else {
                    // 绝对导入：from api.v1.module_system.dict.model import
                    if let Some(target) =
                        resolve_py_absolute_import(module_path, &known_files)
                    {
                        edges.push(DependencyEdge {
                            source: source_path.clone(),
                            target,
                        });
                    }
                }
            }

            // Python import xxx 解析（绝对导入）
            if let Some(caps) = re_py_import.captures(trimmed) {
                let module_path = &caps[1];
                // 排除标准库和第三方包（简单启发式：只匹配项目内存在的路径）
                if let Some(target) =
                    resolve_py_absolute_import(module_path, &known_files)
                {
                    edges.push(DependencyEdge {
                        source: source_path.clone(),
                        target,
                    });
                }
            }
        }
    }

    Ok(edges)
}

/// 判断是否为代码文件（根据扩展名）
fn is_code_file(path: &str) -> bool {
    let code_exts = [
        ".ts", ".tsx", ".js", ".jsx", ".mjs", ".cjs",
        ".py", ".rs", ".vue", ".svelte",
    ];
    code_exts.iter().any(|ext| path.ends_with(ext))
}

/// 解析 JS/TS 相对 import 路径，尝试匹配已知文件
///
/// 尝试顺序：原路径 → 加扩展名 → 加 /index.ts 等
fn resolve_js_import(
    source_dir: &str,
    import_path: &str,
    known_files: &HashSet<&str>,
) -> Option<String> {
    // 拼接为相对于项目根的路径
    let base = if source_dir.is_empty() {
        import_path.to_string()
    } else {
        format!("{}/{}", source_dir, import_path)
    };

    // 规范化路径（处理 ../ 和 ./）
    let normalized = normalize_path(&base);

    // 尝试直接匹配
    if known_files.contains(normalized.as_str()) {
        return Some(normalized);
    }

    // 尝试常见扩展名
    let extensions = [".ts", ".tsx", ".js", ".jsx", ".vue"];
    for ext in &extensions {
        let candidate = format!("{}{}", normalized, ext);
        if known_files.contains(candidate.as_str()) {
            return Some(candidate);
        }
    }

    // 尝试 /index.* 形式
    let index_exts = ["/index.ts", "/index.tsx", "/index.js", "/index.jsx"];
    for idx in &index_exts {
        let candidate = format!("{}{}", normalized, idx);
        if known_files.contains(candidate.as_str()) {
            return Some(candidate);
        }
    }

    None
}

/// 解析 Python 相对 import 路径
///
/// 例如 `from .utils import helper` → 同目录下的 utils.py 或 utils/__init__.py
fn resolve_py_import(
    source_dir: &str,
    module_path: &str,
    known_files: &HashSet<&str>,
) -> Option<String> {
    // 计算前导点数（相对层级）
    let dots = module_path.chars().take_while(|c| *c == '.').count();
    let module_name = &module_path[dots..];

    // 根据点数向上回溯目录
    let mut base_dir = source_dir.to_string();
    for _ in 1..dots {
        if let Some(pos) = base_dir.rfind('/') {
            base_dir = base_dir[..pos].to_string();
        } else {
            base_dir = String::new();
        }
    }

    // 将模块名中的 . 替换为 /
    let module_as_path = module_name.replace('.', "/");

    let base = if base_dir.is_empty() {
        module_as_path
    } else {
        format!("{}/{}", base_dir, module_as_path)
    };

    // 尝试 .py 文件
    let py_file = format!("{}.py", base);
    if known_files.contains(py_file.as_str()) {
        return Some(py_file);
    }

    // 尝试 __init__.py（包目录）
    let init_file = format!("{}/__init__.py", base);
    if known_files.contains(init_file.as_str()) {
        return Some(init_file);
    }

    None
}
/// 解析 Python 绝对 import 路径
///
/// 将点分模块路径（如 `api.v1.module_system.dict.model`）转换为文件路径，
/// 并在已知文件集合中查找匹配。
///
/// 支持场景：
/// - 直接匹配：`api.v1.dict.model` → `api/v1/dict/model.py`
/// - 前缀剥离：当项目根目录本身是包目录时（如项目路径为 `app/`），
///   `app.api.v1.dict.model` 会尝试去掉 `app/` 前缀匹配 `api/v1/dict/model.py`
///
/// 尝试顺序：`{path}.py` → `{path}/__init__.py` → 去掉首段后重试
fn resolve_py_absolute_import(
    module_path: &str,
    known_files: &HashSet<&str>,
) -> Option<String> {
    // 将点号替换为路径分隔符
    let as_path = module_path.replace('.', "/");

    // 尝试直接匹配 .py 文件
    let py_file = format!("{}.py", as_path);
    if known_files.contains(py_file.as_str()) {
        return Some(py_file);
    }

    // 尝试 __init__.py（包目录）
    let init_file = format!("{}/__init__.py", as_path);
    if known_files.contains(init_file.as_str()) {
        return Some(init_file);
    }

    // 前缀剥离策略：当项目根目录本身是包目录时，
    // import 路径的第一段可能是项目根目录名（如 app.xxx → xxx）
    // 逐层去掉前缀尝试匹配
    if let Some(dot_pos) = module_path.find('.') {
        let stripped = &module_path[dot_pos + 1..];
        if !stripped.is_empty() {
            let stripped_path = stripped.replace('.', "/");

            let py_file2 = format!("{}.py", stripped_path);
            if known_files.contains(py_file2.as_str()) {
                return Some(py_file2);
            }

            let init_file2 = format!("{}/__init__.py", stripped_path);
            if known_files.contains(init_file2.as_str()) {
                return Some(init_file2);
            }
        }
    }

    None
}

/// 规范化路径：处理 `.` 和 `..` 段
fn normalize_path(path: &str) -> String {
    let mut parts: Vec<&str> = Vec::new();
    for segment in path.split('/') {
        match segment {
            "." | "" => {}
            ".." => {
                parts.pop();
            }
            _ => parts.push(segment),
        }
    }
    parts.join("/")
}

// ============================================================================
// 向量搜索
// ============================================================================

/// 相似文件搜索结果
#[derive(Debug, Clone)]
pub struct SimilarFileResult {
    /// 文件相对路径
    pub relative_path: String,
    /// 文件摘要
    pub summary: Option<String>,
    /// 余弦相似度分数（0.0 ~ 1.0）
    pub score: f32,
}

/// 计算两个向量的余弦相似度
///
/// 返回值范围 [-1.0, 1.0]，1.0 表示完全相同方向
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }

    let mut dot = 0.0f32;
    let mut norm_a = 0.0f32;
    let mut norm_b = 0.0f32;

    for i in 0..a.len() {
        dot += a[i] * b[i];
        norm_a += a[i] * a[i];
        norm_b += b[i] * b[i];
    }

    let denom = norm_a.sqrt() * norm_b.sqrt();
    if denom == 0.0 {
        0.0
    } else {
        dot / denom
    }
}

/// 将 f32 向量序列化为字节数组（用于存入 SQLite BLOB）
pub fn embedding_to_bytes(embedding: &[f32]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(embedding.len() * 4);
    for &val in embedding {
        bytes.extend_from_slice(&val.to_le_bytes());
    }
    bytes
}

/// 将字节数组反序列化为 f32 向量（从 SQLite BLOB 读取）
pub fn bytes_to_embedding(bytes: &[u8]) -> Vec<f32> {
    bytes
        .chunks_exact(4)
        .map(|chunk| {
            let arr: [u8; 4] = chunk.try_into().unwrap();
            f32::from_le_bytes(arr)
        })
        .collect()
}

// ============================================================================
// 项目概览分析
// ============================================================================

use std::collections::HashMap;
use serde::Serialize;

/// 语言统计条目
#[derive(Debug, Clone, Serialize)]
pub struct LanguageStat {
    /// 语言名称
    pub language: String,
    /// 文件数量
    pub file_count: u32,
    /// 总行数
    pub line_count: u32,
}

/// 项目概览数据
#[derive(Debug, Clone, Serialize)]
pub struct ProjectOverview {
    /// 总文件数
    pub total_files: u32,
    /// 总代码行数
    pub total_lines: u32,
    /// 总目录数
    pub total_dirs: u32,
    /// 检测到的技术栈标签（如 "Python", "FastAPI", "SQLAlchemy"）
    pub tech_stack: Vec<String>,
    /// 按语言分类的文件统计
    pub languages: Vec<LanguageStat>,
    /// 入口文件列表（如 main.py, app.py, index.ts）
    pub entry_files: Vec<String>,
}

/// 分析项目概览信息：技术栈检测、文件统计、语言分布
///
/// 纯文件系统操作，不依赖数据库或 Tauri
pub fn analyze_project_overview(project_path: &Path) -> Result<ProjectOverview, String> {
    if !project_path.exists() {
        return Err(format!("项目路径不存在：{}", project_path.display()));
    }

    // 收集所有文件
    let entries = scan_project_files(project_path)?;

    // 统计目录数
    let dir_set: HashSet<String> = entries.iter().filter_map(|e| {
        let idx = e.relative_path.rfind('/');
        idx.map(|i| e.relative_path[..i].to_string())
    }).collect();
    let total_dirs = dir_set.len() as u32;

    // 按扩展名分组统计语言
    let mut lang_files: HashMap<String, Vec<String>> = HashMap::new();
    for entry in &entries {
        let lang = detect_language(&entry.relative_path);
        lang_files.entry(lang).or_default().push(entry.relative_path.clone());
    }

    // 统计每种语言的行数
    let mut languages: Vec<LanguageStat> = Vec::new();
    let mut total_lines: u32 = 0;

    for (language, files) in &lang_files {
        let mut file_count = 0u32;
        let mut line_count = 0u32;
        for file_path in files {
            let abs_path = project_path.join(file_path);
            if let Ok(content) = std::fs::read_to_string(&abs_path) {
                line_count += content.lines().count() as u32;
                file_count += 1;
            } else {
                file_count += 1; // 二进制文件也计数
            }
        }
        total_lines += line_count;
        languages.push(LanguageStat {
            language: language.clone(),
            file_count,
            line_count,
        });
    }

    // 按行数降序排序
    languages.sort_by(|a, b| b.line_count.cmp(&a.line_count));

    // 检测技术栈
    let tech_stack = detect_tech_stack(project_path, &entries);

    // 检测入口文件
    let entry_files = detect_entry_files(&entries);

    Ok(ProjectOverview {
        total_files: entries.len() as u32,
        total_lines,
        total_dirs,
        tech_stack,
        languages,
        entry_files,
    })
}

/// 根据文件扩展名检测语言
fn detect_language(path: &str) -> String {
    let ext = path.rsplit('.').next().unwrap_or("").to_lowercase();
    match ext.as_str() {
        "py" => "Python".to_string(),
        "js" => "JavaScript".to_string(),
        "ts" => "TypeScript".to_string(),
        "tsx" => "TypeScript (React)".to_string(),
        "jsx" => "JavaScript (React)".to_string(),
        "vue" => "Vue".to_string(),
        "rs" => "Rust".to_string(),
        "go" => "Go".to_string(),
        "java" => "Java".to_string(),
        "kt" | "kts" => "Kotlin".to_string(),
        "rb" => "Ruby".to_string(),
        "php" => "PHP".to_string(),
        "cs" => "C#".to_string(),
        "cpp" | "cc" | "cxx" => "C++".to_string(),
        "c" | "h" => "C".to_string(),
        "swift" => "Swift".to_string(),
        "html" | "htm" => "HTML".to_string(),
        "css" => "CSS".to_string(),
        "scss" | "sass" => "SCSS".to_string(),
        "less" => "Less".to_string(),
        "json" => "JSON".to_string(),
        "yaml" | "yml" => "YAML".to_string(),
        "toml" => "TOML".to_string(),
        "xml" => "XML".to_string(),
        "sql" => "SQL".to_string(),
        "sh" | "bash" => "Shell".to_string(),
        "md" | "markdown" => "Markdown".to_string(),
        "txt" => "Text".to_string(),
        "ini" | "cfg" | "conf" => "Config".to_string(),
        "dockerfile" => "Dockerfile".to_string(),
        _ => "Other".to_string(),
    }
}

/// 检测项目技术栈（通过特征文件和依赖配置）
fn detect_tech_stack(project_path: &Path, entries: &[FileEntry]) -> Vec<String> {
    let mut stack = Vec::new();
    let file_set: HashSet<&str> = entries.iter().map(|e| e.relative_path.as_str()).collect();

    // Python 生态
    let has_py = entries.iter().any(|e| e.relative_path.ends_with(".py"));
    if has_py {
        stack.push("Python".to_string());
    }

    // 检测 requirements.txt / pyproject.toml / setup.py 中的框架
    for config_file in &["requirements.txt", "pyproject.toml", "setup.py", "Pipfile"] {
        let path = project_path.join(config_file);
        if let Ok(content) = std::fs::read_to_string(&path) {
            let lower = content.to_lowercase();
            if lower.contains("fastapi") { push_unique(&mut stack, "FastAPI"); }
            if lower.contains("django") { push_unique(&mut stack, "Django"); }
            if lower.contains("flask") { push_unique(&mut stack, "Flask"); }
            if lower.contains("sqlalchemy") { push_unique(&mut stack, "SQLAlchemy"); }
            if lower.contains("pydantic") { push_unique(&mut stack, "Pydantic"); }
            if lower.contains("celery") { push_unique(&mut stack, "Celery"); }
            if lower.contains("redis") { push_unique(&mut stack, "Redis"); }
            if lower.contains("pytest") { push_unique(&mut stack, "Pytest"); }
            if lower.contains("alembic") { push_unique(&mut stack, "Alembic"); }
            if lower.contains("uvicorn") { push_unique(&mut stack, "Uvicorn"); }
        }
    }

    // JavaScript/TypeScript 生态
    let has_js_ts = entries.iter().any(|e| {
        e.relative_path.ends_with(".ts") || e.relative_path.ends_with(".js")
            || e.relative_path.ends_with(".tsx") || e.relative_path.ends_with(".jsx")
    });
    if has_js_ts {
        // 检测 package.json
        let pkg_path = project_path.join("package.json");
        if let Ok(content) = std::fs::read_to_string(&pkg_path) {
            let lower = content.to_lowercase();
            if lower.contains("\"react\"") { push_unique(&mut stack, "React"); }
            if lower.contains("\"vue\"") { push_unique(&mut stack, "Vue"); }
            if lower.contains("\"next\"") { push_unique(&mut stack, "Next.js"); }
            if lower.contains("\"nuxt\"") { push_unique(&mut stack, "Nuxt"); }
            if lower.contains("\"typescript\"") { push_unique(&mut stack, "TypeScript"); }
            if lower.contains("\"vite\"") { push_unique(&mut stack, "Vite"); }
            if lower.contains("\"tailwindcss\"") { push_unique(&mut stack, "Tailwind CSS"); }
            if lower.contains("\"express\"") { push_unique(&mut stack, "Express"); }
            if lower.contains("\"nestjs\"") || lower.contains("\"@nestjs") { push_unique(&mut stack, "NestJS"); }
        }
    }

    // Rust 生态
    if file_set.contains("Cargo.toml") || project_path.join("Cargo.toml").exists() {
        push_unique(&mut stack, "Rust");
        if let Ok(content) = std::fs::read_to_string(project_path.join("Cargo.toml")) {
            let lower = content.to_lowercase();
            if lower.contains("tauri") { push_unique(&mut stack, "Tauri"); }
            if lower.contains("actix") { push_unique(&mut stack, "Actix"); }
            if lower.contains("tokio") { push_unique(&mut stack, "Tokio"); }
        }
    }

    // Go 生态
    if file_set.contains("go.mod") || project_path.join("go.mod").exists() {
        push_unique(&mut stack, "Go");
    }

    // Java 生态
    if file_set.contains("pom.xml") || project_path.join("pom.xml").exists() {
        push_unique(&mut stack, "Java");
        push_unique(&mut stack, "Maven");
    }
    if file_set.contains("build.gradle") || project_path.join("build.gradle").exists() {
        push_unique(&mut stack, "Java");
        push_unique(&mut stack, "Gradle");
    }

    // Docker
    if file_set.iter().any(|f| f.contains("Dockerfile") || f.contains("dockerfile"))
        || project_path.join("Dockerfile").exists()
        || project_path.join("docker-compose.yml").exists()
    {
        push_unique(&mut stack, "Docker");
    }

    stack
}

/// 辅助：去重添加
fn push_unique(vec: &mut Vec<String>, val: &str) {
    if !vec.iter().any(|v| v == val) {
        vec.push(val.to_string());
    }
}

/// 检测常见入口文件
/// 文件签名提取结果
#[derive(Debug, Clone, Serialize)]
pub struct FileSignature {
    /// 文件相对路径
    pub relative_path: String,
    /// 检测到的语言
    pub language: String,
    /// 提取的签名列表
    pub signatures: Vec<String>,
}

/// 从单个文件内容中提取代码签名（函数、类、接口等）
///
/// 纯本地静态分析，零 API 调用。
/// 支持 Python / JS / TS / Rust / Vue 等语言。
pub fn extract_signatures_from_content(content: &str, language: &str) -> Vec<String> {
    let mut sigs = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty()
            || trimmed.starts_with("//")
            || trimmed.starts_with('#')
            || trimmed.starts_with("/*")
            || trimmed.starts_with('*')
        {
            continue;
        }
        match language {
            "Python" => extract_python_sig(trimmed, &mut sigs),
            "JavaScript" | "TypeScript" | "TSX" | "JSX" => extract_js_sig(trimmed, &mut sigs),
            "Rust" => extract_rust_sig(trimmed, &mut sigs),
            "Vue" => extract_vue_sig(trimmed, &mut sigs),
            _ => extract_generic_sig(trimmed, &mut sigs),
        }
    }
    sigs
}

/// Python 签名提取
fn extract_python_sig(trimmed: &str, sigs: &mut Vec<String>) {
    if trimmed.starts_with("class ") {
        if let Some(name) = trimmed
            .strip_prefix("class ")
            .and_then(|s| s.split(|c: char| c == ':' || c == '(').next())
        {
            sigs.push(format!("class {}", name.trim()));
        }
    } else if trimmed.starts_with("def ") || trimmed.starts_with("async def ") {
        let sig_line = if trimmed.starts_with("async ") {
            &trimmed[6..]
        } else {
            trimmed
        };
        if let Some(paren_end) = sig_line.find(')') {
            sigs.push(sig_line[..paren_end + 1].to_string());
        } else {
            sigs.push(
                sig_line
                    .split(':')
                    .next()
                    .unwrap_or(sig_line)
                    .trim()
                    .to_string(),
            );
        }
    } else if trimmed.starts_with("from ") || trimmed.starts_with("import ") {
        sigs.push(trimmed.to_string());
    }
}

/// JS/TS 签名提取
fn extract_js_sig(trimmed: &str, sigs: &mut Vec<String>) {
    if trimmed.starts_with("export function ")
        || trimmed.starts_with("export async function ")
        || trimmed.starts_with("export default function ")
    {
        if let Some(paren_end) = trimmed.find(')') {
            sigs.push(trimmed[..paren_end + 1].to_string());
        }
    } else if trimmed.starts_with("export class ")
        || trimmed.starts_with("export interface ")
        || trimmed.starts_with("export type ")
        || trimmed.starts_with("export enum ")
    {
        let sig = trimmed.split('{').next().unwrap_or(trimmed).trim();
        sigs.push(sig.to_string());
    } else if trimmed.starts_with("export const ") || trimmed.starts_with("export let ") {
        let sig = trimmed.split('=').next().unwrap_or(trimmed).trim();
        sigs.push(sig.to_string());
    } else if trimmed.starts_with("import ") {
        sigs.push(trimmed.to_string());
    } else if trimmed.starts_with("function ") || trimmed.starts_with("async function ") {
        if let Some(paren_end) = trimmed.find(')') {
            sigs.push(trimmed[..paren_end + 1].to_string());
        }
    } else if trimmed.starts_with("class ") || trimmed.starts_with("interface ") {
        let sig = trimmed.split('{').next().unwrap_or(trimmed).trim();
        sigs.push(sig.to_string());
    }
}

/// Rust 签名提取
fn extract_rust_sig(trimmed: &str, sigs: &mut Vec<String>) {
    if trimmed.starts_with("pub fn ")
        || trimmed.starts_with("pub async fn ")
        || trimmed.starts_with("fn ")
        || trimmed.starts_with("async fn ")
    {
        if let Some(paren_end) = trimmed.find(')') {
            let rest = &trimmed[paren_end + 1..];
            if let Some(brace) = rest.find('{') {
                sigs.push(format!(
                    "{}{}",
                    &trimmed[..paren_end + 1],
                    rest[..brace].trim()
                ));
            } else {
                sigs.push(trimmed[..paren_end + 1].to_string());
            }
        }
    } else if trimmed.starts_with("pub struct ")
        || trimmed.starts_with("struct ")
        || trimmed.starts_with("pub enum ")
        || trimmed.starts_with("enum ")
        || trimmed.starts_with("pub trait ")
        || trimmed.starts_with("trait ")
        || trimmed.starts_with("impl ")
    {
        let sig = trimmed.split('{').next().unwrap_or(trimmed).trim();
        sigs.push(sig.to_string());
    } else if trimmed.starts_with("use ") {
        sigs.push(trimmed.to_string());
    } else if trimmed.starts_with("pub mod ") || trimmed.starts_with("mod ") {
        sigs.push(trimmed.trim_end_matches('{').trim().to_string());
    }
}

/// Vue SFC 签名提取
fn extract_vue_sig(trimmed: &str, sigs: &mut Vec<String>) {
    if trimmed.starts_with("export default")
        || trimmed.starts_with("import ")
        || trimmed.starts_with("export function ")
        || trimmed.starts_with("export const ")
    {
        let sig = trimmed.split('{').next().unwrap_or(trimmed).trim();
        sigs.push(sig.to_string());
    } else if trimmed.starts_with("const ") && trimmed.contains("defineComponent") {
        sigs.push(
            trimmed
                .split('=')
                .next()
                .unwrap_or(trimmed)
                .trim()
                .to_string(),
        );
    }
}

/// 通用签名提取
fn extract_generic_sig(trimmed: &str, sigs: &mut Vec<String>) {
    if trimmed.starts_with("function ") || trimmed.starts_with("class ") {
        let sig = trimmed.split('{').next().unwrap_or(trimmed).trim();
        sigs.push(sig.to_string());
    }
}

/// 批量提取项目所有文件的签名
pub fn extract_project_signatures(project_path: &Path) -> Result<Vec<FileSignature>, String> {
    let entries = scan_project_files(project_path)?;
    let mut results = Vec::new();
    for entry in &entries {
        let lang = detect_language(&entry.relative_path);
        if lang == "Other" {
            continue;
        }
        let full_path = project_path.join(&entry.relative_path);
        let content = match std::fs::read_to_string(&full_path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let sigs = extract_signatures_from_content(&content, &lang);
        if !sigs.is_empty() {
            results.push(FileSignature {
                relative_path: entry.relative_path.clone(),
                language: lang,
                signatures: sigs,
            });
        }
    }
    Ok(results)
}

/// 将签名列表格式化为 LLM 可读的文本
pub fn format_signatures_for_llm(signatures: &[FileSignature]) -> String {
    let mut output = String::new();
    for sig in signatures {
        output.push_str(&format!(
            "[{}] {} | {}\n",
            sig.language,
            sig.relative_path,
            sig.signatures.join(", ")
        ));
    }
    output
}


fn detect_entry_files(entries: &[FileEntry]) -> Vec<String> {
    let entry_patterns = [
        "main.py", "app.py", "manage.py", "wsgi.py", "asgi.py",
        "index.ts", "index.js", "main.ts", "main.js", "app.ts", "app.js",
        "main.rs", "lib.rs",
        "main.go",
        "Main.java", "Application.java",
    ];
    let mut found = Vec::new();
    for entry in entries {
        let filename = entry.relative_path.rsplit('/').next().unwrap_or(&entry.relative_path);
        if entry_patterns.contains(&filename) {
            found.push(entry.relative_path.clone());
        }
    }
    found
}



#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_scan_empty_dir() {
        let tmp = TempDir::new().unwrap();
        let entries = scan_project_files(tmp.path()).unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn test_scan_with_files() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("main.py"), "print('hello')").unwrap();
        fs::create_dir(tmp.path().join("utils")).unwrap();
        fs::write(tmp.path().join("utils/helper.py"), "def help(): pass").unwrap();

        let entries = scan_project_files(tmp.path()).unwrap();
        assert_eq!(entries.len(), 2);

        let paths: Vec<&str> = entries.iter().map(|e| e.relative_path.as_str()).collect();
        assert!(paths.contains(&"main.py"));
        assert!(paths.contains(&"utils/helper.py"));
    }

    #[test]
    fn test_ignored_dirs_filtered() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("app.py"), "pass").unwrap();
        // 创建应被忽略的目录
        fs::create_dir(tmp.path().join("node_modules")).unwrap();
        fs::write(tmp.path().join("node_modules/pkg.js"), "module").unwrap();
        fs::create_dir(tmp.path().join(".git")).unwrap();
        fs::write(tmp.path().join(".git/config"), "git").unwrap();

        let entries = scan_project_files(tmp.path()).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].relative_path, "app.py");
    }

    #[test]
    fn test_hash_consistency() {
        let tmp = TempDir::new().unwrap();
        let content = "hello world";
        fs::write(tmp.path().join("test.txt"), content).unwrap();

        let entries = scan_project_files(tmp.path()).unwrap();
        assert_eq!(entries.len(), 1);

        // 相同内容应产生相同哈希
        let hash1 = &entries[0].file_hash;
        let entries2 = scan_project_files(tmp.path()).unwrap();
        assert_eq!(hash1, &entries2[0].file_hash);
    }

    #[test]
    fn test_nonexistent_path() {
        let result = scan_project_files(Path::new("/nonexistent/path/xyz"));
        assert!(result.is_err());
    }

    // ====================================================================
    // 依赖推断测试
    // ====================================================================

    #[test]
    fn test_extract_js_import_relative() {
        let tmp = TempDir::new().unwrap();
        // 创建源文件和目标文件
        fs::create_dir_all(tmp.path().join("src/components")).unwrap();
        fs::write(
            tmp.path().join("src/App.tsx"),
            "import { Button } from './components/Button';\nimport React from 'react';\n",
        )
        .unwrap();
        fs::write(tmp.path().join("src/components/Button.tsx"), "export function Button() {}").unwrap();

        let file_paths = vec![
            "src/App.tsx".to_string(),
            "src/components/Button.tsx".to_string(),
        ];

        let edges = extract_dependencies(tmp.path(), &file_paths).unwrap();
        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0].source, "src/App.tsx");
        assert_eq!(edges[0].target, "src/components/Button.tsx");
    }

    #[test]
    fn test_extract_js_import_ignores_packages() {
        let tmp = TempDir::new().unwrap();
        fs::write(
            tmp.path().join("index.ts"),
            "import React from 'react';\nimport { toast } from 'sonner';\n",
        )
        .unwrap();

        let file_paths = vec!["index.ts".to_string()];
        let edges = extract_dependencies(tmp.path(), &file_paths).unwrap();
        // 第三方包不应产生边
        assert!(edges.is_empty());
    }

    #[test]
    fn test_extract_js_import_parent_dir() {
        let tmp = TempDir::new().unwrap();
        fs::create_dir_all(tmp.path().join("src/pages")).unwrap();
        fs::write(
            tmp.path().join("src/pages/Home.tsx"),
            "import { store } from '../store';\n",
        )
        .unwrap();
        fs::write(tmp.path().join("src/store.ts"), "export const store = {};").unwrap();

        let file_paths = vec![
            "src/pages/Home.tsx".to_string(),
            "src/store.ts".to_string(),
        ];

        let edges = extract_dependencies(tmp.path(), &file_paths).unwrap();
        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0].target, "src/store.ts");
    }

    #[test]
    fn test_extract_js_import_index_resolution() {
        let tmp = TempDir::new().unwrap();
        fs::create_dir_all(tmp.path().join("src/components")).unwrap();
        fs::write(
            tmp.path().join("src/App.tsx"),
            "import { Foo } from './components';\n",
        )
        .unwrap();
        fs::write(tmp.path().join("src/components/index.ts"), "export const Foo = 1;").unwrap();

        let file_paths = vec![
            "src/App.tsx".to_string(),
            "src/components/index.ts".to_string(),
        ];

        let edges = extract_dependencies(tmp.path(), &file_paths).unwrap();
        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0].target, "src/components/index.ts");
    }

    #[test]
    fn test_extract_python_relative_import() {
        let tmp = TempDir::new().unwrap();
        fs::create_dir_all(tmp.path().join("app")).unwrap();
        fs::write(
            tmp.path().join("app/main.py"),
            "from .utils import helper\n",
        )
        .unwrap();
        fs::write(tmp.path().join("app/utils.py"), "def helper(): pass").unwrap();

        let file_paths = vec![
            "app/main.py".to_string(),
            "app/utils.py".to_string(),
        ];

        let edges = extract_dependencies(tmp.path(), &file_paths).unwrap();
        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0].source, "app/main.py");
        assert_eq!(edges[0].target, "app/utils.py");
    }

    #[test]
    fn test_extract_fastapi_project_full() {
        // 模拟完整的 FastAPI 项目结构，验证绝对导入依赖提取
        let tmp = TempDir::new().unwrap();

        // 创建目录结构
        fs::create_dir_all(tmp.path().join("api/v1/module_system/dict")).unwrap();
        fs::create_dir_all(tmp.path().join("api/v1/module_system/user")).unwrap();
        fs::create_dir_all(tmp.path().join("core")).unwrap();
        fs::create_dir_all(tmp.path().join("utils")).unwrap();

        // 创建文件
        fs::write(tmp.path().join("api/__init__.py"), "").unwrap();
        fs::write(tmp.path().join("api/v1/__init__.py"), "").unwrap();
        fs::write(tmp.path().join("api/v1/module_system/__init__.py"), "").unwrap();
        fs::write(tmp.path().join("api/v1/module_system/dict/__init__.py"), "").unwrap();
        fs::write(
            tmp.path().join("api/v1/module_system/dict/controller.py"),
            "from api.v1.module_system.dict.model import DictModel\nfrom api.v1.module_system.dict.schema import DictCreate\nfrom core.database import get_db\nfrom fastapi import APIRouter\n",
        ).unwrap();
        fs::write(tmp.path().join("api/v1/module_system/dict/model.py"), "class DictModel: pass").unwrap();
        fs::write(tmp.path().join("api/v1/module_system/dict/schema.py"), "class DictCreate: pass").unwrap();
        fs::write(tmp.path().join("api/v1/module_system/user/__init__.py"), "").unwrap();
        fs::write(
            tmp.path().join("api/v1/module_system/user/controller.py"),
            "from api.v1.module_system.user.model import UserModel\nimport api.v1.module_system.dict.model\n",
        ).unwrap();
        fs::write(tmp.path().join("api/v1/module_system/user/model.py"), "class UserModel: pass").unwrap();
        fs::write(tmp.path().join("core/__init__.py"), "").unwrap();
        fs::write(tmp.path().join("core/database.py"), "def get_db(): pass").unwrap();
        fs::write(tmp.path().join("utils/__init__.py"), "").unwrap();
        fs::write(tmp.path().join("utils/string_util.py"), "def to_camel(): pass").unwrap();

        // 扫描文件
        let entries = scan_project_files(tmp.path()).unwrap();
        let file_paths: Vec<String> = entries.iter().map(|e| e.relative_path.clone()).collect();

        println!("扫描到 {} 个文件:", file_paths.len());
        for p in &file_paths {
            println!("  {}", p);
        }

        // 提取依赖
        let edges = extract_dependencies(tmp.path(), &file_paths).unwrap();

        println!("\n提取到 {} 条依赖:", edges.len());
        for e in &edges {
            println!("  {} -> {}", e.source, e.target);
        }

        // 验证：dict/controller.py 应该有 3 条依赖（model, schema, core/database）
        let dict_ctrl_deps: Vec<&DependencyEdge> = edges
            .iter()
            .filter(|e| e.source == "api/v1/module_system/dict/controller.py")
            .collect();
        assert!(
            dict_ctrl_deps.len() >= 2,
            "dict/controller.py 应至少有 2 条项目内依赖，实际 {}",
            dict_ctrl_deps.len()
        );

        // 验证：user/controller.py 应该有 2 条依赖（user/model, dict/model）
        let user_ctrl_deps: Vec<&DependencyEdge> = edges
            .iter()
            .filter(|e| e.source == "api/v1/module_system/user/controller.py")
            .collect();
        assert!(
            user_ctrl_deps.len() >= 2,
            "user/controller.py 应至少有 2 条项目内依赖，实际 {}",
            user_ctrl_deps.len()
        );

        // 总依赖数应 > 0
        assert!(edges.len() > 0, "应该提取到依赖，但实际为 0");
    }

    #[test]
    fn test_extract_python_absolute_from_import() {
        // 测试 Python 绝对导入：from api.v1.dict.model import DictModel
        let tmp = TempDir::new().unwrap();
        fs::create_dir_all(tmp.path().join("api/v1/dict")).unwrap();
        fs::write(
            tmp.path().join("api/v1/dict/controller.py"),
            "from api.v1.dict.model import DictModel\nfrom api.v1.dict.schema import DictSchema\n",
        )
        .unwrap();
        fs::write(tmp.path().join("api/v1/dict/model.py"), "class DictModel: pass").unwrap();
        fs::write(tmp.path().join("api/v1/dict/schema.py"), "class DictSchema: pass").unwrap();

        let file_paths = vec![
            "api/v1/dict/controller.py".to_string(),
            "api/v1/dict/model.py".to_string(),
            "api/v1/dict/schema.py".to_string(),
        ];

        let edges = extract_dependencies(tmp.path(), &file_paths).unwrap();
        assert_eq!(edges.len(), 2);
        assert_eq!(edges[0].source, "api/v1/dict/controller.py");
        assert_eq!(edges[0].target, "api/v1/dict/model.py");
        assert_eq!(edges[1].source, "api/v1/dict/controller.py");
        assert_eq!(edges[1].target, "api/v1/dict/schema.py");
    }

    #[test]
    fn test_extract_python_absolute_import_statement() {
        // 测试 Python 绝对导入：import api.v1.dict.model
        let tmp = TempDir::new().unwrap();
        fs::create_dir_all(tmp.path().join("api/v1/dict")).unwrap();
        fs::write(
            tmp.path().join("api/v1/dict/controller.py"),
            "import api.v1.dict.model\n",
        )
        .unwrap();
        fs::write(tmp.path().join("api/v1/dict/model.py"), "class DictModel: pass").unwrap();

        let file_paths = vec![
            "api/v1/dict/controller.py".to_string(),
            "api/v1/dict/model.py".to_string(),
        ];

        let edges = extract_dependencies(tmp.path(), &file_paths).unwrap();
        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0].source, "api/v1/dict/controller.py");
        assert_eq!(edges[0].target, "api/v1/dict/model.py");
    }

    #[test]
    fn test_extract_python_absolute_import_package() {
        // 测试 Python 绝对导入到包目录（__init__.py）
        let tmp = TempDir::new().unwrap();
        fs::create_dir_all(tmp.path().join("api/v1/dict")).unwrap();
        fs::write(
            tmp.path().join("api/v1/dict/controller.py"),
            "from api.v1.dict import something\n",
        )
        .unwrap();
        fs::write(tmp.path().join("api/v1/dict/__init__.py"), "something = 1").unwrap();

        let file_paths = vec![
            "api/v1/dict/controller.py".to_string(),
            "api/v1/dict/__init__.py".to_string(),
        ];

        let edges = extract_dependencies(tmp.path(), &file_paths).unwrap();
        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0].target, "api/v1/dict/__init__.py");
    }

    #[test]
    fn test_extract_python_absolute_import_with_prefix() {
        // 测试项目根目录本身是包目录的场景
        // 项目路径为 app/，import 写法为 from app.api.v1.dict.model import ...
        // known_files 中路径为 api/v1/dict/model.py（不含 app/ 前缀）
        let tmp = TempDir::new().unwrap();
        fs::create_dir_all(tmp.path().join("api/v1/module_system/config")).unwrap();
        fs::create_dir_all(tmp.path().join("common")).unwrap();
        fs::create_dir_all(tmp.path().join("core")).unwrap();

        fs::write(
            tmp.path().join("api/v1/module_system/config/controller.py"),
            "from app.api.v1.module_system.config.param import ConfigQueryParams\nfrom app.common.request import PaginationService\nfrom app.api.v1.module_system.config.service import ConfigService\nfrom fastapi import APIRouter\n",
        ).unwrap();
        fs::write(tmp.path().join("api/v1/module_system/config/param.py"), "class ConfigQueryParams: pass").unwrap();
        fs::write(tmp.path().join("api/v1/module_system/config/service.py"), "class ConfigService: pass").unwrap();
        fs::write(tmp.path().join("common/request.py"), "class PaginationService: pass").unwrap();
        fs::write(tmp.path().join("core/dependencies.py"), "pass").unwrap();

        let file_paths = vec![
            "api/v1/module_system/config/controller.py".to_string(),
            "api/v1/module_system/config/param.py".to_string(),
            "api/v1/module_system/config/service.py".to_string(),
            "common/request.py".to_string(),
            "core/dependencies.py".to_string(),
        ];

        let edges = extract_dependencies(tmp.path(), &file_paths).unwrap();

        println!("提取到 {} 条依赖:", edges.len());
        for e in &edges {
            println!("  {} -> {}", e.source, e.target);
        }

        // controller.py 应该匹配到 param.py, service.py, common/request.py（通过前缀剥离）
        assert!(edges.len() >= 3, "应至少有 3 条依赖（前缀剥离匹配），实际 {}", edges.len());

        let targets: Vec<&str> = edges.iter().map(|e| e.target.as_str()).collect();
        assert!(targets.contains(&"api/v1/module_system/config/param.py"), "应匹配到 param.py");
        assert!(targets.contains(&"api/v1/module_system/config/service.py"), "应匹配到 service.py");
        assert!(targets.contains(&"common/request.py"), "应匹配到 common/request.py");
    }

    #[test]
    fn test_extract_python_absolute_import_ignores_third_party() {
        // 绝对导入如果在项目文件中找不到，应被忽略（第三方包）
        let tmp = TempDir::new().unwrap();
        fs::write(
            tmp.path().join("main.py"),
            "import os\nimport sys\nfrom fastapi import FastAPI\n",
        )
        .unwrap();

        let file_paths = vec!["main.py".to_string()];
        let edges = extract_dependencies(tmp.path(), &file_paths).unwrap();
        assert!(edges.is_empty());
    }

    #[test]
    fn test_extract_skips_comment_lines() {
        // 注释行中的 import 应被跳过
        let tmp = TempDir::new().unwrap();
        fs::create_dir_all(tmp.path().join("app")).unwrap();
        fs::write(
            tmp.path().join("app/main.py"),
            "# from .utils import helper\n// import something\nfrom .utils import helper\n",
        )
        .unwrap();
        fs::write(tmp.path().join("app/utils.py"), "def helper(): pass").unwrap();

        let file_paths = vec![
            "app/main.py".to_string(),
            "app/utils.py".to_string(),
        ];

        let edges = extract_dependencies(tmp.path(), &file_paths).unwrap();
        // 只有第三行的非注释 import 应被解析
        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0].target, "app/utils.py");
    }

    #[test]
    fn test_extract_skips_non_code_files() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("readme.md"), "import something from './foo';\n").unwrap();
        fs::write(tmp.path().join("data.json"), "{}").unwrap();

        let file_paths = vec![
            "readme.md".to_string(),
            "data.json".to_string(),
        ];

        let edges = extract_dependencies(tmp.path(), &file_paths).unwrap();
        assert!(edges.is_empty());
    }

    #[test]
    fn test_normalize_path() {
        assert_eq!(normalize_path("src/./utils/../store.ts"), "src/store.ts");
        assert_eq!(normalize_path("./components/Button"), "components/Button");
        assert_eq!(normalize_path("a/b/../../c"), "c");
    }

    // ====================================================================
    // 向量搜索测试
    // ====================================================================

    #[test]
    fn test_cosine_similarity_identical() {
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![1.0, 2.0, 3.0];
        let sim = cosine_similarity(&a, &b);
        assert!((sim - 1.0).abs() < 1e-6, "相同向量相似度应为 1.0");
    }

    #[test]
    fn test_cosine_similarity_orthogonal() {
        let a = vec![1.0, 0.0];
        let b = vec![0.0, 1.0];
        let sim = cosine_similarity(&a, &b);
        assert!(sim.abs() < 1e-6, "正交向量相似度应为 0.0");
    }

    #[test]
    fn test_cosine_similarity_opposite() {
        let a = vec![1.0, 2.0];
        let b = vec![-1.0, -2.0];
        let sim = cosine_similarity(&a, &b);
        assert!((sim + 1.0).abs() < 1e-6, "反向向量相似度应为 -1.0");
    }

    #[test]
    fn test_cosine_similarity_empty() {
        let a: Vec<f32> = vec![];
        let b: Vec<f32> = vec![];
        assert_eq!(cosine_similarity(&a, &b), 0.0);
    }

    #[test]
    fn test_cosine_similarity_different_lengths() {
        let a = vec![1.0, 2.0];
        let b = vec![1.0, 2.0, 3.0];
        assert_eq!(cosine_similarity(&a, &b), 0.0);
    }

    #[test]
    fn test_embedding_roundtrip() {
        let original = vec![0.1, -0.5, 3.14, 0.0, -1.0];
        let bytes = embedding_to_bytes(&original);
        let restored = bytes_to_embedding(&bytes);
        assert_eq!(original.len(), restored.len());
        for (a, b) in original.iter().zip(restored.iter()) {
            assert!((a - b).abs() < 1e-7, "序列化/反序列化应保持精度");
        }
    }

    #[test]
    fn test_embedding_bytes_length() {
        let emb = vec![1.0f32; 768]; // 常见 embedding 维度
        let bytes = embedding_to_bytes(&emb);
        assert_eq!(bytes.len(), 768 * 4); // 每个 f32 占 4 字节
    }
}
