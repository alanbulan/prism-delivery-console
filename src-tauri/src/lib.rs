// ============================================================================
// [总线] 程序的组装车间
// ✅ 只能做：pub mod 暴露子模块、注册 .invoke_handler()、初始化 State
// ⛔ 禁止：直接实现 command 函数
// ============================================================================

pub mod commands;
pub mod database;
pub mod models;
pub mod services;
pub mod utils;

use tauri::Manager;

// ============================================================================
// 应用入口
// ============================================================================

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            // 获取应用数据目录并初始化数据库
            let app_data_dir = app
                .path()
                .app_data_dir()
                .map_err(|e| format!("获取应用数据目录失败: {}", e))?;
            let db = database::Database::init(&app_data_dir)
                .map_err(|e| Box::new(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
            // 注册数据库为 Tauri managed state（使用 Mutex 保证线程安全）
            app.manage(std::sync::Mutex::new(db));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // 项目 commands
            commands::project::open_project,
            commands::project::scan_modules,
            commands::project::scan_project_modules,
            // 构建 commands
            commands::build::build_package,
            commands::build::build_project_package,
            commands::build::open_folder,
            // 数据库 CRUD commands
            commands::db_crud::db_create_category,
            commands::db_crud::db_list_categories,
            commands::db_crud::db_update_category,
            commands::db_crud::db_delete_category,
            commands::db_crud::db_create_project,
            commands::db_crud::db_list_projects,
            commands::db_crud::db_update_project,
            commands::db_crud::db_delete_project,
            commands::db_crud::db_create_client,
            commands::db_crud::db_list_clients_by_project,
            commands::db_crud::db_update_client,
            commands::db_crud::db_delete_client,
            commands::db_crud::db_create_build_record,
            commands::db_crud::db_list_build_records,
            // 设置 commands
            commands::db_crud::get_app_settings,
            commands::db_crud::save_app_setting,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
