// ============================================================================
// 数据库 CRUD Commands
// 作为前端与数据库层之间的薄接口层，仅负责：
// 1. 接收前端参数
// 2. 从 Tauri State 获取 Database 实例
// 3. 调用 Database 方法
// 4. 返回结果
// ⛔ 禁止：包含业务逻辑
// ============================================================================

use crate::database::{BuildRecord, Category, Client, Database, Project};
use std::sync::Mutex;
use tauri::State;

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
) -> Result<Project, String> {
    let db = db
        .lock()
        .map_err(|_| "数据库访问失败：无法获取锁".to_string())?;
    db.create_project(&name, category_id, &repo_path, &tech_stack)
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
    tech_stack: String,
) -> Result<(), String> {
    let db = db
        .lock()
        .map_err(|_| "数据库访问失败：无法获取锁".to_string())?;
    db.update_project(id, &name, category_id, &tech_stack)
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
) -> Result<BuildRecord, String> {
    let db = db
        .lock()
        .map_err(|_| "数据库访问失败：无法获取锁".to_string())?;
    db.create_build_record(project_id, client_id, &modules_json, &output_path)
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
