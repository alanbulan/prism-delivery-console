// ============================================================================
// 项目分析相关 Commands
// 负责：LLM 配置管理、模型列表获取、文件索引
// ✅ 只能做：接收前端参数、简单校验、调用 services 层、返回 Result
// ⛔ 禁止：写文件读写、数据库操作、复杂算法
// ============================================================================

use crate::database::Database;
use crate::services::{analyzer, llm_client};
use serde::Serialize;
use std::sync::Mutex;
use tauri::State;

/// LLM 配置（从 settings 表读取，返回给前端）
#[derive(Serialize)]
pub struct LlmConfig {
    pub base_url: String,
    pub api_key: String,
    pub model_name: String,
    pub embedding_model: String,
}

/// LLM 模型信息（返回给前端）
#[derive(Serialize)]
pub struct LlmModel {
    pub id: String,
}

/// 获取 LLM 配置
///
/// 从 settings 表中读取 llm_base_url、llm_api_key、llm_model_name 三个键值
#[tauri::command]
pub fn get_llm_config(db: State<'_, Mutex<Database>>) -> Result<LlmConfig, String> {
    let db = db.lock().map_err(|e| format!("数据库锁获取失败：{}", e))?;
    let conn = db.conn();

    // 辅助函数：从 settings 表读取值，不存在则返回空字符串
    let get_setting = |key: &str| -> String {
        conn.query_row(
            "SELECT value FROM settings WHERE key = ?1",
            rusqlite::params![key],
            |row| row.get::<_, String>(0),
        )
        .unwrap_or_default()
    };

    Ok(LlmConfig {
        base_url: get_setting("llm_base_url"),
        api_key: get_setting("llm_api_key"),
        model_name: get_setting("llm_model_name"),
        embedding_model: get_setting("llm_embedding_model"),
    })
}

/// 从 OpenAI 兼容 API 获取可用模型列表
///
/// # 参数
/// - `base_url`: API 基础地址
/// - `api_key`: API Key（可为空）
#[tauri::command]
pub async fn list_llm_models(base_url: String, api_key: String) -> Result<Vec<LlmModel>, String> {
    // 参数校验
    if base_url.trim().is_empty() {
        return Err("API 基础地址不能为空".to_string());
    }

    // 委托给 services 层
    let model_ids = llm_client::fetch_models(&base_url, &api_key).await?;

    Ok(model_ids.into_iter().map(|id| LlmModel { id }).collect())
}

/// 文件索引条目（返回给前端）
#[derive(Serialize)]
pub struct FileIndexEntry {
    /// 相对路径
    pub relative_path: String,
    /// SHA256 哈希
    pub file_hash: String,
    /// 是否有变更（与数据库中的哈希不同）
    pub changed: bool,
    /// LLM 生成的文件摘要（可为空）
    pub summary: Option<String>,
}

/// 扫描项目文件并与数据库中的索引对比，返回增量变更信息
///
/// # 参数
/// - `project_id`: 项目 ID（用于查询/更新 file_index 表）
/// - `project_path`: 项目根目录路径
#[tauri::command]
pub fn scan_project_file_index(
    db: State<'_, Mutex<Database>>,
    project_id: i64,
    project_path: String,
) -> Result<Vec<FileIndexEntry>, String> {
    // 调用 services 层扫描文件（含 file_size + mtime 元数据）
    let entries =
        analyzer::scan_project_files(std::path::Path::new(&project_path))?;

    let db = db.lock().map_err(|e| format!("数据库锁获取失败：{}", e))?;
    let conn = db.conn();

    // 从数据库加载已有的文件索引（含 file_size、mtime 用于增量快速判断）
    let mut existing: std::collections::HashMap<String, (String, Option<String>, u64, u64)> =
        std::collections::HashMap::new();
    {
        let mut stmt = conn
            .prepare("SELECT file_path, file_hash, summary, file_size, mtime FROM file_index WHERE project_id = ?1")
            .map_err(|e| format!("查询文件索引失败：{}", e))?;
        let rows = stmt
            .query_map(rusqlite::params![project_id], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<String>>(2)?,
                    row.get::<_, u64>(3).unwrap_or(0),
                    row.get::<_, u64>(4).unwrap_or(0),
                ))
            })
            .map_err(|e| format!("查询文件索引失败：{}", e))?;
        for row in rows {
            let (path, hash, summary, size, mtime) =
                row.map_err(|e| format!("读取文件索引失败：{}", e))?;
            existing.insert(path, (hash, summary, size, mtime));
        }
    }

    // 增量对比：先用 file_size + mtime 快速判断，跳过未变化文件的哈希比较
    let mut result = Vec::with_capacity(entries.len());
    for entry in &entries {
        let (changed, old_summary, effective_hash) = match existing.get(&entry.relative_path) {
            Some((old_hash, summary, old_size, old_mtime)) => {
                // 快速路径：文件大小和修改时间都未变，直接复用缓存哈希
                if *old_size == entry.file_size && *old_mtime == entry.mtime {
                    (false, summary.clone(), old_hash.clone())
                } else {
                    // 元数据变化，用新哈希对比
                    let hash_changed = old_hash != &entry.file_hash;
                    let kept_summary = if hash_changed { None } else { summary.clone() };
                    (hash_changed, kept_summary, entry.file_hash.clone())
                }
            }
            None => (true, None, entry.file_hash.clone()), // 新文件视为变更
        };

        // 使用 UPSERT 更新文件索引（含 file_size、mtime）
        conn.execute(
            "INSERT INTO file_index (project_id, file_path, file_hash, summary, file_size, mtime, last_analyzed_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, datetime('now'))
             ON CONFLICT(project_id, file_path)
             DO UPDATE SET file_hash = ?3, summary = ?4, file_size = ?5, mtime = ?6, last_analyzed_at = datetime('now')",
            rusqlite::params![
                project_id,
                entry.relative_path,
                effective_hash,
                if changed { None::<String> } else { old_summary.clone() },
                entry.file_size as i64,
                entry.mtime as i64,
            ],
        )
        .map_err(|e| format!("更新文件索引失败：{}", e))?;

        result.push(FileIndexEntry {
            relative_path: entry.relative_path.clone(),
            file_hash: effective_hash,
            changed,
            summary: old_summary,
        });
    }

    // 清理数据库中已不存在的文件记录
    let current_paths: std::collections::HashSet<&str> =
        entries.iter().map(|e| e.relative_path.as_str()).collect();
    for old_path in existing.keys() {
        if !current_paths.contains(old_path.as_str()) {
            conn.execute(
                "DELETE FROM file_index WHERE project_id = ?1 AND file_path = ?2",
                rusqlite::params![project_id, old_path],
            )
            .map_err(|e| format!("清理文件索引失败：{}", e))?;
        }
    }

    Ok(result)
}


/// 为单个文件生成 LLM 摘要并存入数据库
///
/// # 参数
/// - `project_id`: 项目 ID
/// - `project_path`: 项目根目录路径
/// - `file_path`: 文件相对路径
#[tauri::command]
pub async fn analyze_file_summary(
    db: State<'_, Mutex<Database>>,
    project_id: i64,
    project_path: String,
    file_path: String,
) -> Result<String, String> {
    // 1. 从 settings 表读取 LLM 配置
    let (base_url, api_key, model_name) = {
        let db = db.lock().map_err(|e| format!("数据库锁获取失败：{}", e))?;
        let conn = db.conn();
        let get = |key: &str| -> String {
            conn.query_row(
                "SELECT value FROM settings WHERE key = ?1",
                rusqlite::params![key],
                |row| row.get::<_, String>(0),
            )
            .unwrap_or_default()
        };
        (get("llm_base_url"), get("llm_api_key"), get("llm_model_name"))
    };

    if base_url.is_empty() || model_name.is_empty() {
        return Err("请先在设置页面配置 LLM API 地址和模型".to_string());
    }

    // 2. 路径安全校验：防止路径遍历攻击
    if file_path.contains("..") {
        return Err(format!("非法文件路径（包含 ..）: {}", file_path));
    }

    // 3. 读取文件内容
    let abs_path = std::path::Path::new(&project_path).join(&file_path);
    let content = std::fs::read_to_string(&abs_path)
        .map_err(|e| format!("读取文件失败 {}: {}", file_path, e))?;

    // 3. 调用 LLM 生成摘要
    let summary = llm_client::generate_summary(
        &base_url, &api_key, &model_name, &file_path, &content,
    )
    .await?;

    // 4. 将摘要写入数据库
    {
        let db = db.lock().map_err(|e| format!("数据库锁获取失败：{}", e))?;
        let conn = db.conn();
        conn.execute(
            "UPDATE file_index SET summary = ?1 WHERE project_id = ?2 AND file_path = ?3",
            rusqlite::params![summary, project_id, file_path],
        )
        .map_err(|e| format!("保存摘要失败：{}", e))?;
    }

    Ok(summary)
}

// ============================================================================
// 依赖分析
// ============================================================================

/// 依赖边（返回给前端）
#[derive(Serialize)]
pub struct DepEdge {
    pub source: String,
    pub target: String,
}

/// 依赖图数据（返回给前端）
#[derive(Serialize)]
pub struct DependencyGraph {
    /// 所有文件节点（相对路径）
    pub nodes: Vec<String>,
    /// 依赖边列表
    pub edges: Vec<DepEdge>,
}

/// 分析项目文件间的 import 依赖关系
///
/// # 参数
/// - `project_path`: 项目根目录路径
#[tauri::command]
pub fn analyze_dependencies(project_path: String) -> Result<DependencyGraph, String> {
    let path = std::path::Path::new(&project_path);

    // 1. 扫描项目文件
    let entries = analyzer::scan_project_files(path)?;
    let file_paths: Vec<String> = entries.iter().map(|e| e.relative_path.clone()).collect();

    // 2. 提取依赖关系
    let dep_edges = analyzer::extract_dependencies(path, &file_paths)?;

    // 3. 构建返回数据
    Ok(DependencyGraph {
        nodes: file_paths,
        edges: dep_edges
            .into_iter()
            .map(|e| DepEdge {
                source: e.source,
                target: e.target,
            })
            .collect(),
    })
}

// ============================================================================
// Embedding / 语义搜索
// ============================================================================

/// 为单个文件生成 Embedding 向量并存入数据库
///
/// 使用文件摘要（summary）作为 embedding 输入文本。
/// 如果文件没有摘要，则使用文件路径 + 文件内容前 2000 字符。
///
/// # 参数
/// - `project_id`: 项目 ID
/// - `project_path`: 项目根目录路径
/// - `file_path`: 文件相对路径
#[tauri::command]
pub async fn embed_file(
    db: State<'_, Mutex<Database>>,
    project_id: i64,
    project_path: String,
    file_path: String,
) -> Result<(), String> {
    // 1. 从 settings 表读取 Embedding 配置
    let (base_url, api_key, embed_model) = {
        let db = db.lock().map_err(|e| format!("数据库锁获取失败：{}", e))?;
        let conn = db.conn();
        let get = |key: &str| -> String {
            conn.query_row(
                "SELECT value FROM settings WHERE key = ?1",
                rusqlite::params![key],
                |row| row.get::<_, String>(0),
            )
            .unwrap_or_default()
        };
        (get("llm_base_url"), get("llm_api_key"), get("llm_embedding_model"))
    };

    if base_url.is_empty() || embed_model.is_empty() {
        return Err("请先在设置页面配置 API 地址和 Embedding 模型".to_string());
    }

    // 2. 获取文件摘要或读取文件内容作为 embedding 输入
    let input_text = {
        let db = db.lock().map_err(|e| format!("数据库锁获取失败：{}", e))?;
        let conn = db.conn();
        let summary: Option<String> = conn
            .query_row(
                "SELECT summary FROM file_index WHERE project_id = ?1 AND file_path = ?2",
                rusqlite::params![project_id, file_path],
                |row| row.get(0),
            )
            .unwrap_or(None);

        match summary {
            Some(s) if !s.is_empty() => format!("文件：{}\n摘要：{}", file_path, s),
            _ => {
                // 没有摘要时，使用文件路径 + 内容前 2000 字符
                let abs_path = std::path::Path::new(&project_path).join(&file_path);
                let content = std::fs::read_to_string(&abs_path)
                    .map_err(|e| format!("读取文件失败 {}: {}", file_path, e))?;
                let truncated = if content.len() > 2000 { &content[..2000] } else { &content };
                format!("文件：{}\n内容：{}", file_path, truncated)
            }
        }
    };

    // 3. 调用 Embedding API
    let embedding = llm_client::generate_embedding(
        &base_url, &api_key, &embed_model, &input_text,
    )
    .await?;

    // 4. 序列化并存入数据库
    let bytes = analyzer::embedding_to_bytes(&embedding);
    {
        let db = db.lock().map_err(|e| format!("数据库锁获取失败：{}", e))?;
        let conn = db.conn();
        conn.execute(
            "UPDATE file_index SET embedding = ?1 WHERE project_id = ?2 AND file_path = ?3",
            rusqlite::params![bytes, project_id, file_path],
        )
        .map_err(|e| format!("保存 Embedding 失败：{}", e))?;
    }

    Ok(())
}

/// 批量为项目所有文件生成 Embedding
///
/// # 参数
/// - `project_id`: 项目 ID
/// - `project_path`: 项目根目录路径
///
/// # 返回
/// - 成功生成 embedding 的文件数量
#[tauri::command]
pub async fn embed_all_files(
    db: State<'_, Mutex<Database>>,
    project_id: i64,
    project_path: String,
) -> Result<EmbedBatchResult, String> {
    // 1. 读取配置
    let (base_url, api_key, embed_model) = {
        let db = db.lock().map_err(|e| format!("数据库锁获取失败：{}", e))?;
        let conn = db.conn();
        let get = |key: &str| -> String {
            conn.query_row(
                "SELECT value FROM settings WHERE key = ?1",
                rusqlite::params![key],
                |row| row.get::<_, String>(0),
            )
            .unwrap_or_default()
        };
        (get("llm_base_url"), get("llm_api_key"), get("llm_embedding_model"))
    };

    if base_url.is_empty() || embed_model.is_empty() {
        return Err("请先在设置页面配置 API 地址和 Embedding 模型".to_string());
    }

    // 2. 获取所有缺少 embedding 的文件
    let files_to_embed: Vec<(String, Option<String>)> = {
        let db = db.lock().map_err(|e| format!("数据库锁获取失败：{}", e))?;
        let conn = db.conn();
        let mut stmt = conn
            .prepare(
                "SELECT file_path, summary FROM file_index WHERE project_id = ?1 AND embedding IS NULL",
            )
            .map_err(|e| format!("查询文件索引失败：{}", e))?;
        let rows = stmt
            .query_map(rusqlite::params![project_id], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, Option<String>>(1)?))
            })
            .map_err(|e| format!("查询文件索引失败：{}", e))?;
        rows.filter_map(|r| r.ok()).collect()
    };

    let total = files_to_embed.len();
    let mut success_count = 0u32;
    let mut fail_count = 0u32;

    // 3. 逐个生成 embedding
    for (file_path, summary) in &files_to_embed {
        let input_text = match summary {
            Some(s) if !s.is_empty() => format!("文件：{}\n摘要：{}", file_path, s),
            _ => {
                let abs_path = std::path::Path::new(&project_path).join(file_path);
                match std::fs::read_to_string(&abs_path) {
                    Ok(content) => {
                        let truncated = if content.len() > 2000 { &content[..2000] } else { &content };
                        format!("文件：{}\n内容：{}", file_path, truncated)
                    }
                    Err(_) => {
                        fail_count += 1;
                        continue;
                    }
                }
            }
        };

        match llm_client::generate_embedding(&base_url, &api_key, &embed_model, &input_text).await {
            Ok(embedding) => {
                let bytes = analyzer::embedding_to_bytes(&embedding);
                let db = db.lock().map_err(|e| format!("数据库锁获取失败：{}", e))?;
                let conn = db.conn();
                conn.execute(
                    "UPDATE file_index SET embedding = ?1 WHERE project_id = ?2 AND file_path = ?3",
                    rusqlite::params![bytes, project_id, file_path],
                )
                .map_err(|e| format!("保存 Embedding 失败：{}", e))?;
                success_count += 1;
            }
            Err(e) => {
                // 记录具体失败原因，便于排查
                log::warn!("Embedding 生成失败 [{}]: {}", file_path, e);
                fail_count += 1;
            }
        }
    }

    Ok(EmbedBatchResult {
        total: total as u32,
        success: success_count,
        failed: fail_count,
    })
}

/// 批量 Embedding 结果
#[derive(Serialize)]
pub struct EmbedBatchResult {
    pub total: u32,
    pub success: u32,
    pub failed: u32,
}

/// 语义搜索：根据查询文本找到最相似的文件
///
/// # 参数
/// - `project_id`: 项目 ID
/// - `query`: 搜索查询文本
/// - `top_k`: 返回前 K 个最相似的结果
#[tauri::command]
pub async fn search_similar_files(
    db: State<'_, Mutex<Database>>,
    project_id: i64,
    query: String,
    top_k: usize,
) -> Result<Vec<SimilarFileEntry>, String> {
    // 1. 读取配置
    let (base_url, api_key, embed_model) = {
        let db = db.lock().map_err(|e| format!("数据库锁获取失败：{}", e))?;
        let conn = db.conn();
        let get = |key: &str| -> String {
            conn.query_row(
                "SELECT value FROM settings WHERE key = ?1",
                rusqlite::params![key],
                |row| row.get::<_, String>(0),
            )
            .unwrap_or_default()
        };
        (get("llm_base_url"), get("llm_api_key"), get("llm_embedding_model"))
    };

    if base_url.is_empty() || embed_model.is_empty() {
        return Err("请先在设置页面配置 API 地址和 Embedding 模型".to_string());
    }

    // 2. 生成查询文本的 embedding
    let query_embedding = llm_client::generate_embedding(
        &base_url, &api_key, &embed_model, &query,
    )
    .await?;

    // 3. 从数据库加载所有有 embedding 的文件
    let file_embeddings: Vec<(String, Option<String>, Vec<u8>)> = {
        let db = db.lock().map_err(|e| format!("数据库锁获取失败：{}", e))?;
        let conn = db.conn();
        let mut stmt = conn
            .prepare(
                "SELECT file_path, summary, embedding FROM file_index WHERE project_id = ?1 AND embedding IS NOT NULL",
            )
            .map_err(|e| format!("查询文件索引失败：{}", e))?;
        let rows = stmt
            .query_map(rusqlite::params![project_id], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, Option<String>>(1)?,
                    row.get::<_, Vec<u8>>(2)?,
                ))
            })
            .map_err(|e| format!("查询文件索引失败：{}", e))?;
        rows.filter_map(|r| r.ok()).collect()
    };

    if file_embeddings.is_empty() {
        return Ok(vec![]);
    }

    // 4. 计算余弦相似度并排序
    let mut results: Vec<SimilarFileEntry> = file_embeddings
        .iter()
        .map(|(path, summary, bytes)| {
            let emb = analyzer::bytes_to_embedding(bytes);
            let score = analyzer::cosine_similarity(&query_embedding, &emb);
            SimilarFileEntry {
                relative_path: path.clone(),
                summary: summary.clone(),
                score,
            }
        })
        .collect();

    // 按相似度降序排序
    results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

    // 取 Top-K
    results.truncate(top_k);

    Ok(results)
}

/// 语义搜索结果条目（返回给前端）
#[derive(Serialize)]
pub struct SimilarFileEntry {
    /// 文件相对路径
    pub relative_path: String,
    /// 文件摘要
    pub summary: Option<String>,
    /// 余弦相似度分数
    pub score: f32,
}

// ============================================================================
// 项目概览
// ============================================================================

/// 语言统计条目（返回给前端）
#[derive(Serialize)]
pub struct LanguageStatEntry {
    pub language: String,
    pub file_count: u32,
    pub line_count: u32,
}

/// 项目概览数据（返回给前端）
#[derive(Serialize)]
pub struct ProjectOverviewEntry {
    pub total_files: u32,
    pub total_lines: u32,
    pub total_dirs: u32,
    pub tech_stack: Vec<String>,
    pub languages: Vec<LanguageStatEntry>,
    pub entry_files: Vec<String>,
}

/// 获取项目概览信息（技术栈检测、文件统计、语言分布）
///
/// # 参数
/// - `project_path`: 项目根目录路径
#[tauri::command]
pub fn get_project_overview(project_path: String) -> Result<ProjectOverviewEntry, String> {
    let path = std::path::Path::new(&project_path);
    let overview = analyzer::analyze_project_overview(path)?;

    Ok(ProjectOverviewEntry {
        total_files: overview.total_files,
        total_lines: overview.total_lines,
        total_dirs: overview.total_dirs,
        tech_stack: overview.tech_stack,
        languages: overview.languages.into_iter().map(|l| LanguageStatEntry {
            language: l.language,
            file_count: l.file_count,
            line_count: l.line_count,
        }).collect(),
        entry_files: overview.entry_files,
    })
}

// ============================================================================
// 签名索引 + 报告生成
// ============================================================================

/// 签名索引结果（返回给前端）
#[derive(Serialize)]
pub struct IndexSignaturesResult {
    /// 总文件数
    pub total: u32,
    /// 成功提取签名的文件数
    pub indexed: u32,
}

/// 后台提取项目所有文件的静态签名并存入数据库
///
/// # 参数
/// - `project_id`: 项目 ID
/// - `project_path`: 项目根目录路径
#[tauri::command]
pub fn index_project_signatures(
    db: State<'_, Mutex<Database>>,
    project_id: i64,
    project_path: String,
) -> Result<IndexSignaturesResult, String> {
    let path = std::path::Path::new(&project_path);

    // 1. 提取所有文件签名
    let signatures = analyzer::extract_project_signatures(path)?;
    let total = signatures.len() as u32;

    // 2. 将签名序列化后存入 file_index.signatures 列
    let db = db.lock().map_err(|e| format!("数据库锁获取失败：{}", e))?;
    let conn = db.conn();

    let mut indexed = 0u32;
    for sig in &signatures {
        let sig_json = serde_json::to_string(&sig.signatures)
            .unwrap_or_else(|_| "[]".to_string());
        let rows = conn.execute(
            "UPDATE file_index SET signatures = ?1 WHERE project_id = ?2 AND file_path = ?3",
            rusqlite::params![sig_json, project_id, sig.relative_path],
        ).map_err(|e| format!("更新签名失败：{}", e))?;
        if rows > 0 {
            indexed += 1;
        }
    }

    Ok(IndexSignaturesResult { total, indexed })
}

/// 生成项目分析报告（收集签名+概览+依赖，调用 LLM）
///
/// # 参数
/// - `project_id`: 项目 ID
/// - `project_path`: 项目根目录路径
/// - `mode`: 报告模式 "fast"（1次LLM调用）或 "deep"（分层压缩）
#[tauri::command]
pub async fn generate_project_report(
    db: State<'_, Mutex<Database>>,
    _project_id: i64,
    project_path: String,
    mode: String,
) -> Result<String, String> {
    let path = std::path::Path::new(&project_path);

    // 1. 读取 LLM 配置
    let (base_url, api_key, model_name) = {
        let db = db.lock().map_err(|e| format!("数据库锁获取失败：{}", e))?;
        let conn = db.conn();
        let get = |key: &str| -> String {
            conn.query_row(
                "SELECT value FROM settings WHERE key = ?1",
                rusqlite::params![key],
                |row| row.get::<_, String>(0),
            )
            .unwrap_or_default()
        };
        (get("llm_base_url"), get("llm_api_key"), get("llm_model_name"))
    };

    if base_url.is_empty() || model_name.is_empty() {
        return Err("请先在设置页面配置 LLM API 地址和模型".to_string());
    }

    // 2. 收集项目数据
    let overview = analyzer::analyze_project_overview(path)?;
    let signatures = analyzer::extract_project_signatures(path)?;
    let sig_text = analyzer::format_signatures_for_llm(&signatures);

    // 3. 收集依赖关系
    let entries = analyzer::scan_project_files(path)?;
    let file_paths: Vec<String> = entries.iter().map(|e| e.relative_path.clone()).collect();
    let dep_edges = analyzer::extract_dependencies(path, &file_paths)?;
    let dep_text = dep_edges
        .iter()
        .take(200) // 限制依赖边数量，避免 prompt 过长
        .map(|e| format!("  {} -> {}", e.source, e.target))
        .collect::<Vec<_>>()
        .join("\n");

    // 4. 构建 system prompt
    let system_prompt = "你是一个资深软件架构师。请根据提供的项目数据，生成一份全面的项目分析报告。\n\
        报告使用 Markdown 格式，包含以下章节：\n\
        1. 项目概述（技术栈、规模）\n\
        2. 架构分析（模块划分、分层结构）\n\
        3. 核心模块详解（关键文件和函数的职责）\n\
        4. 依赖关系分析（模块间耦合度、循环依赖风险）\n\
        5. 代码质量评估（命名规范、复杂度、可维护性）\n\
        6. 改进建议（架构优化、重构方向）\n\
        请用中文撰写，分析要深入具体，不要泛泛而谈。";

    // 5. 构建 user prompt
    let lang_text = overview.languages.iter()
        .map(|l| format!("- {}：{} 文件，{} 行", l.language, l.file_count, l.line_count))
        .collect::<Vec<_>>()
        .join("\n");

    let user_prompt = format!(
        "## 项目统计\n- 文件数：{}\n- 代码行数：{}\n- 目录数：{}\n- 技术栈：{}\n- 入口文件：{}\n\n\
         ## 语言分布\n{}\n\n\
         ## 代码签名（类/函数/接口声明）\n{}\n\n\
         ## 依赖关系（source -> target）\n{}",
        overview.total_files,
        overview.total_lines,
        overview.total_dirs,
        overview.tech_stack.join(", "),
        overview.entry_files.join(", "),
        lang_text,
        sig_text,
        dep_text,
    );

    // 6. 根据模式调用 LLM
    match mode.as_str() {
        "fast" => {
            // Fast 模式：直接一次调用
            llm_client::generate_report(
                &base_url, &api_key, &model_name,
                system_prompt, &user_prompt,
            ).await
        }
        "deep" => {
            // Deep 模式：签名过长时先压缩再汇总
            if sig_text.len() > 30000 {
                // 第一步：压缩签名摘要
                let compress_prompt = format!(
                    "以下是一个大型项目的代码签名列表，请将其压缩为一份结构化摘要，\
                    保留关键的类、函数和模块信息，去除重复和不重要的细节：\n\n{}",
                    sig_text
                );
                let compressed = llm_client::generate_report(
                    &base_url, &api_key, &model_name,
                    "你是一个代码分析助手，请压缩以下代码签名信息。",
                    &compress_prompt,
                ).await?;

                // 第二步：用压缩后的签名生成报告
                let final_prompt = format!(
                    "## 项目统计\n- 文件数：{}\n- 代码行数：{}\n- 目录数：{}\n- 技术栈：{}\n\n\
                     ## 代码结构摘要\n{}\n\n\
                     ## 依赖关系\n{}",
                    overview.total_files,
                    overview.total_lines,
                    overview.total_dirs,
                    overview.tech_stack.join(", "),
                    compressed,
                    dep_text,
                );
                llm_client::generate_report(
                    &base_url, &api_key, &model_name,
                    system_prompt, &final_prompt,
                ).await
            } else {
                // 签名不多，等同于 fast 模式
                llm_client::generate_report(
                    &base_url, &api_key, &model_name,
                    system_prompt, &user_prompt,
                ).await
            }
        }
        _ => Err(format!("不支持的报告模式：{}", mode)),
    }
}

