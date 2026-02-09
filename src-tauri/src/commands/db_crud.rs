// ============================================================================
// 数据库 CRUD Commands
// 作为前端与数据库层之间的薄接口层，仅负责：
// 1. 接收前端参数
// 2. 从 Tauri State 获取 Database 实例
// 3. 调用 Database 方法
// 4. 返回结果
// ⛔ 禁止：包含业务逻辑
// ============================================================================

use crate::database::{BuildRecord, Category, Client, Database, Project, TechStackTemplate};
use std::sync::Mutex;
use tauri::State;

// ============================================================================
// 辅助函数
// ============================================================================

/// 删除构建记录对应的 ZIP 文件（尽力删除，失败仅记录日志不阻断流程）
fn delete_output_files(records: &[BuildRecord]) {
    for record in records {
        let path = std::path::Path::new(&record.output_path);
        if path.exists() {
            if let Err(e) = std::fs::remove_file(path) {
                log::warn!("删除构建文件失败（已忽略）：{} - {}", record.output_path, e);
            } else {
                log::info!("已删除构建文件：{}", record.output_path);
            }
        }
    }
}

// ============================================================================
// 分类 CRUD Commands
// ============================================================================

/// 创建分类
#[tauri::command]
pub async fn db_create_category(
    db: State<'_, Mutex<Database>>,
    name: String,
    description: Option<String>,
) -> Result<Category, String> {
    let db = db
        .lock()
        .map_err(|_| "数据库访问失败：无法获取锁".to_string())?;
    db.create_category(&name, description.as_deref())
}

/// 查询所有分类
#[tauri::command]
pub async fn db_list_categories(db: State<'_, Mutex<Database>>) -> Result<Vec<Category>, String> {
    let db = db
        .lock()
        .map_err(|_| "数据库访问失败：无法获取锁".to_string())?;
    db.list_categories()
}

/// 更新分类
#[tauri::command]
pub async fn db_update_category(
    db: State<'_, Mutex<Database>>,
    id: i64,
    name: String,
    description: Option<String>,
) -> Result<(), String> {
    let db = db
        .lock()
        .map_err(|_| "数据库访问失败：无法获取锁".to_string())?;
    db.update_category(id, &name, description.as_deref())
}

/// 删除分类
#[tauri::command]
pub async fn db_delete_category(db: State<'_, Mutex<Database>>, id: i64) -> Result<(), String> {
    let db = db
        .lock()
        .map_err(|_| "数据库访问失败：无法获取锁".to_string())?;
    db.delete_category(id)
}

// ============================================================================
// 项目 CRUD Commands
// ============================================================================

/// 创建项目
#[tauri::command]
pub async fn db_create_project(
    db: State<'_, Mutex<Database>>,
    name: String,
    category_id: i64,
    repo_path: String,
    tech_stack: String,
    modules_dir: String,
) -> Result<Project, String> {
    let db = db
        .lock()
        .map_err(|_| "数据库访问失败：无法获取锁".to_string())?;
    db.create_project(&name, category_id, &repo_path, &tech_stack, &modules_dir)
}

/// 查询所有项目
#[tauri::command]
pub async fn db_list_projects(db: State<'_, Mutex<Database>>) -> Result<Vec<Project>, String> {
    let db = db
        .lock()
        .map_err(|_| "数据库访问失败：无法获取锁".to_string())?;
    db.list_projects()
}

/// 更新项目
#[tauri::command]
pub async fn db_update_project(
    db: State<'_, Mutex<Database>>,
    id: i64,
    name: String,
    category_id: i64,
    repo_path: String,
    tech_stack: String,
    modules_dir: String,
) -> Result<(), String> {
    let db = db
        .lock()
        .map_err(|_| "数据库访问失败：无法获取锁".to_string())?;
    db.update_project(id, &name, category_id, &repo_path, &tech_stack, &modules_dir)
}

/// 删除项目
#[tauri::command]
pub async fn db_delete_project(db: State<'_, Mutex<Database>>, id: i64) -> Result<(), String> {
    let db = db
        .lock()
        .map_err(|_| "数据库访问失败：无法获取锁".to_string())?;
    db.delete_project(id)
}

// ============================================================================
// 客户 CRUD Commands
// ============================================================================

/// 创建客户并关联到指定项目
#[tauri::command]
pub async fn db_create_client(
    db: State<'_, Mutex<Database>>,
    name: String,
    project_ids: Vec<i64>,
) -> Result<Client, String> {
    let db = db
        .lock()
        .map_err(|_| "数据库访问失败：无法获取锁".to_string())?;
    db.create_client(&name, &project_ids)
}

/// 查询指定项目关联的所有客户
#[tauri::command]
pub async fn db_list_clients_by_project(
    db: State<'_, Mutex<Database>>,
    project_id: i64,
) -> Result<Vec<Client>, String> {
    let db = db
        .lock()
        .map_err(|_| "数据库访问失败：无法获取锁".to_string())?;
    db.list_clients_by_project(project_id)
}

/// 更新客户名称
#[tauri::command]
pub async fn db_update_client(
    db: State<'_, Mutex<Database>>,
    id: i64,
    name: String,
) -> Result<(), String> {
    let db = db
        .lock()
        .map_err(|_| "数据库访问失败：无法获取锁".to_string())?;
    db.update_client(id, &name)
}

/// 删除客户
#[tauri::command]
pub async fn db_delete_client(db: State<'_, Mutex<Database>>, id: i64) -> Result<(), String> {
    let db = db
        .lock()
        .map_err(|_| "数据库访问失败：无法获取锁".to_string())?;
    db.delete_client(id)
}

// ============================================================================
// 构建记录 Commands
// ============================================================================

/// 创建构建记录
#[tauri::command]
pub async fn db_create_build_record(
    db: State<'_, Mutex<Database>>,
    project_id: i64,
    client_id: i64,
    modules_json: String,
    output_path: String,
    version: String,
    changelog: Option<String>,
) -> Result<BuildRecord, String> {
    let db = db
        .lock()
        .map_err(|_| "数据库访问失败：无法获取锁".to_string())?;
    db.create_build_record(project_id, client_id, &modules_json, &output_path, &version, changelog.as_deref())
}

/// 查询指定项目的构建记录列表
#[tauri::command]
pub async fn db_list_build_records(
    db: State<'_, Mutex<Database>>,
    project_id: i64,
) -> Result<Vec<BuildRecord>, String> {
    let db = db
        .lock()
        .map_err(|_| "数据库访问失败：无法获取锁".to_string())?;
    db.list_build_records_by_project(project_id)
}

/// 删除单条构建记录
/// - `delete_files`: 是否同时删除对应的 ZIP 文件
#[tauri::command]
pub async fn db_delete_build_record(
    db: State<'_, Mutex<Database>>,
    id: i64,
    delete_files: bool,
) -> Result<(), String> {
    let db = db
        .lock()
        .map_err(|_| "数据库访问失败：无法获取锁".to_string())?;

    // 如果需要删除文件，先查出记录的 output_path
    if delete_files {
        if let Ok(records) = db.list_build_records_by_ids(&[id]) {
            delete_output_files(&records);
        }
    }

    db.delete_build_record(id)
}

/// 清空指定项目的所有构建记录
/// - `delete_files`: 是否同时删除对应的 ZIP 文件
#[tauri::command]
pub async fn db_delete_all_build_records(
    db: State<'_, Mutex<Database>>,
    project_id: i64,
    delete_files: bool,
) -> Result<u64, String> {
    let db = db
        .lock()
        .map_err(|_| "数据库访问失败：无法获取锁".to_string())?;

    // 如果需要删除文件，先查出所有记录的 output_path
    if delete_files {
        if let Ok(records) = db.list_build_records_by_project(project_id) {
            delete_output_files(&records);
        }
    }

    db.delete_all_build_records(project_id)
}

/// 删除指定项目中 N 天前的构建记录
/// - `delete_files`: 是否同时删除对应的 ZIP 文件
#[tauri::command]
pub async fn db_delete_build_records_before_days(
    db: State<'_, Mutex<Database>>,
    project_id: i64,
    days: i64,
    delete_files: bool,
) -> Result<u64, String> {
    let db = db
        .lock()
        .map_err(|_| "数据库访问失败：无法获取锁".to_string())?;

    // 如果需要删除文件，先查出符合条件的记录的 output_path
    if delete_files {
        if let Ok(records) = db.list_build_records_before_days(project_id, days) {
            delete_output_files(&records);
        }
    }

    db.delete_build_records_before_days(project_id, days)
}


// ============================================================================
// 设置 Commands
// ============================================================================

/// 获取应用设置
#[tauri::command]
pub async fn get_app_settings(
    db: State<'_, Mutex<Database>>,
) -> Result<crate::database::AppSettings, String> {
    let db = db
        .lock()
        .map_err(|_| "数据库访问失败：无法获取锁".to_string())?;
    let db_path = db.conn().path().map(|p| p.to_string()).unwrap_or_default();
    db.get_settings(&db_path)
}

/// 读取单个设置项
#[tauri::command]
pub async fn get_app_setting(
    db: State<'_, Mutex<Database>>,
    key: String,
) -> Result<Option<String>, String> {
    let db = db
        .lock()
        .map_err(|_| "数据库访问失败：无法获取锁".to_string())?;
    db.get_setting(&key)
}

/// 保存单个设置项
#[tauri::command]
pub async fn save_app_setting(
    db: State<'_, Mutex<Database>>,
    key: String,
    value: String,
) -> Result<(), String> {
    let db = db
        .lock()
        .map_err(|_| "数据库访问失败：无法获取锁".to_string())?;
    db.save_setting(&key, &value)
}

// ============================================================================
// 客户模块配置 Commands
// ============================================================================

/// 保存客户模块配置（记忆客户在某项目下选择的模块）
#[tauri::command]
pub async fn db_save_client_modules(
    db: State<'_, Mutex<Database>>,
    client_id: i64,
    project_id: i64,
    modules_json: String,
) -> Result<(), String> {
    let db = db
        .lock()
        .map_err(|_| "数据库访问失败：无法获取锁".to_string())?;
    db.save_client_module_config(client_id, project_id, &modules_json)
}

/// 加载客户模块配置
#[tauri::command]
pub async fn db_load_client_modules(
    db: State<'_, Mutex<Database>>,
    client_id: i64,
    project_id: i64,
) -> Result<Option<String>, String> {
    let db = db
        .lock()
        .map_err(|_| "数据库访问失败：无法获取锁".to_string())?;
    db.load_client_module_config(client_id, project_id)
}

/// 获取下一个构建版本号
#[tauri::command]
pub async fn db_get_next_version(
    db: State<'_, Mutex<Database>>,
    client_id: i64,
    project_id: i64,
) -> Result<String, String> {
    let db = db
        .lock()
        .map_err(|_| "数据库访问失败：无法获取锁".to_string())?;
    db.get_next_version(client_id, project_id)
}

/// 获取该客户在该项目下最近一次构建的模块列表
#[tauri::command]
pub async fn db_get_last_build_modules(
    db: State<'_, Mutex<Database>>,
    client_id: i64,
    project_id: i64,
) -> Result<Option<String>, String> {
    let db = db
        .lock()
        .map_err(|_| "数据库访问失败：无法获取锁".to_string())?;
    db.get_last_build_modules(client_id, project_id)
}

// ============================================================================
// 技术栈模板 Commands
// ============================================================================

/// 创建自定义技术栈模板
#[tauri::command]
pub async fn db_create_template(
    db: State<'_, Mutex<Database>>,
    name: String,
    modules_dir: String,
    extra_excludes: String,
    entry_file: String,
    import_pattern: String,
    router_pattern: String,
) -> Result<TechStackTemplate, String> {
    let db = db
        .lock()
        .map_err(|_| "数据库访问失败：无法获取锁".to_string())?;
    db.create_template(&name, &modules_dir, &extra_excludes, &entry_file, &import_pattern, &router_pattern)
}

/// 查询所有技术栈模板
#[tauri::command]
pub async fn db_list_templates(
    db: State<'_, Mutex<Database>>,
) -> Result<Vec<TechStackTemplate>, String> {
    let db = db
        .lock()
        .map_err(|_| "数据库访问失败：无法获取锁".to_string())?;
    db.list_templates()
}

/// 更新自定义技术栈模板（内置模板不可修改）
#[tauri::command]
pub async fn db_update_template(
    db: State<'_, Mutex<Database>>,
    id: i64,
    name: String,
    modules_dir: String,
    extra_excludes: String,
    entry_file: String,
    import_pattern: String,
    router_pattern: String,
) -> Result<(), String> {
    let db = db
        .lock()
        .map_err(|_| "数据库访问失败：无法获取锁".to_string())?;
    db.update_template(id, &name, &modules_dir, &extra_excludes, &entry_file, &import_pattern, &router_pattern)
}

/// 删除自定义技术栈模板（内置模板不可删除）
#[tauri::command]
pub async fn db_delete_template(
    db: State<'_, Mutex<Database>>,
    id: i64,
) -> Result<(), String> {
    let db = db
        .lock()
        .map_err(|_| "数据库访问失败：无法获取锁".to_string())?;
    db.delete_template(id)
}

/// 导出模板为 JSON 字符串（用于分享/备份）
#[tauri::command]
pub async fn export_template_json(
    db: State<'_, Mutex<Database>>,
    id: i64,
) -> Result<String, String> {
    let db = db
        .lock()
        .map_err(|_| "数据库访问失败：无法获取锁".to_string())?;
    let templates = db.list_templates()?;
    let template = templates
        .into_iter()
        .find(|t| t.id == id)
        .ok_or_else(|| format!("模板不存在：ID {}", id))?;
    serde_json::to_string_pretty(&template)
        .map_err(|e| format!("序列化模板失败：{}", e))
}

/// 从 JSON 字符串导入模板（创建新的自定义模板）
#[tauri::command]
pub async fn import_template_json(
    db: State<'_, Mutex<Database>>,
    json_str: String,
) -> Result<TechStackTemplate, String> {
    // 反序列化 JSON，提取字段创建新模板
    let imported: TechStackTemplate = serde_json::from_str(&json_str)
        .map_err(|e| format!("JSON 格式错误：{}", e))?;
    let db = db
        .lock()
        .map_err(|_| "数据库访问失败：无法获取锁".to_string())?;
    db.create_template(
        &imported.name,
        &imported.modules_dir,
        &imported.extra_excludes,
        &imported.entry_file,
        &imported.import_pattern,
        &imported.router_pattern,
    )
}
