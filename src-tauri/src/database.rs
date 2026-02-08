// ============================================================================
// 数据库模块：SQLite 持久化层
// 使用 rusqlite 直接操作 SQLite，遵循 KISS 原则，不引入 ORM
// ============================================================================

use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::Path;

// ============================================================================
// 数据结构定义
// ============================================================================

/// 项目分类
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Category {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub created_at: String,
}

/// 项目信息
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Project {
    pub id: i64,
    pub name: String,
    pub category_id: i64,
    pub repo_path: String,
    pub tech_stack_type: String,
    pub created_at: String,
    pub updated_at: String,
}

/// 交付客户
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Client {
    pub id: i64,
    pub name: String,
    pub created_at: String,
}

/// 构建记录
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BuildRecord {
    pub id: i64,
    pub project_id: i64,
    pub client_id: i64,
    /// JSON 数组格式的模块列表
    pub selected_modules: String,
    pub output_path: String,
    pub created_at: String,
}

/// 应用设置
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AppSettings {
    pub default_output_dir: Option<String>,
    pub db_path: String,
}

// ============================================================================
// 数据库管理器
// ============================================================================

/// 数据库管理器，封装 rusqlite 连接
pub struct Database {
    /// SQLite 数据库连接
    conn: Connection,
}

impl Database {
    /// 初始化数据库：在指定目录创建数据库文件并建表
    ///
    /// # 参数
    /// - `app_data_dir`: 应用数据目录路径（Tauri app_data_dir）
    ///
    /// # 返回
    /// - `Ok(Database)`: 初始化成功，返回数据库实例
    /// - `Err(String)`: 初始化失败，返回中文错误描述
    pub fn init(app_data_dir: &Path) -> Result<Self, String> {
        // 确保数据目录存在
        std::fs::create_dir_all(app_data_dir).map_err(|e| {
            format!(
                "数据库初始化失败：无法创建数据目录 {}: {}",
                app_data_dir.display(),
                e
            )
        })?;

        // 在数据目录下创建/打开数据库文件
        let db_path = app_data_dir.join("prism_console.db");
        let conn = Connection::open(&db_path).map_err(|e| {
            format!(
                "数据库初始化失败：无法打开数据库文件 {}: {}",
                db_path.display(),
                e
            )
        })?;

        // 启用外键约束（SQLite 默认关闭外键支持）
        conn.execute_batch("PRAGMA foreign_keys = ON;")
            .map_err(|e| format!("数据库初始化失败：无法启用外键约束: {}", e))?;

        // 创建所有必要的表
        Self::create_tables(&conn)?;

        Ok(Database { conn })
    }

    /// 创建所有数据库表（如果不存在）
    ///
    /// 按照设计文档 Data Models 部分定义的 Schema 创建六张表：
    /// categories, projects, clients, project_clients, build_records, settings
    fn create_tables(conn: &Connection) -> Result<(), String> {
        conn.execute_batch(
            "
            -- 分类表
            CREATE TABLE IF NOT EXISTS categories (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                description TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            -- 项目表
            CREATE TABLE IF NOT EXISTS projects (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                category_id INTEGER NOT NULL,
                repo_path TEXT NOT NULL,
                tech_stack_type TEXT NOT NULL DEFAULT 'fastapi',
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY (category_id) REFERENCES categories(id)
            );

            -- 客户表
            CREATE TABLE IF NOT EXISTS clients (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            -- 项目-客户关联表（多对多）
            CREATE TABLE IF NOT EXISTS project_clients (
                project_id INTEGER NOT NULL,
                client_id INTEGER NOT NULL,
                PRIMARY KEY (project_id, client_id),
                FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
                FOREIGN KEY (client_id) REFERENCES clients(id) ON DELETE CASCADE
            );

            -- 构建记录表
            CREATE TABLE IF NOT EXISTS build_records (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                project_id INTEGER NOT NULL,
                client_id INTEGER NOT NULL,
                selected_modules TEXT NOT NULL,
                output_path TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
                FOREIGN KEY (client_id) REFERENCES clients(id)
            );

            -- 设置表（键值对）
            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );
            ",
        )
        .map_err(|e| format!("数据库初始化失败：创建表结构时出错: {}", e))?;

        Ok(())
    }

    /// 获取数据库连接的引用（供 CRUD 方法使用）
    pub fn conn(&self) -> &Connection {
        &self.conn
    }

    // ========================================================================
    // 分类 CRUD 方法
    // ========================================================================

    /// 创建分类
    ///
    /// # 参数
    /// - `name`: 分类名称（必须唯一）
    /// - `description`: 可选的分类描述
    ///
    /// # 返回
    /// - `Ok(Category)`: 创建成功，返回完整的分类记录
    /// - `Err(String)`: 创建失败（如名称重复），返回中文错误描述
    pub fn create_category(
        &self,
        name: &str,
        description: Option<&str>,
    ) -> Result<Category, String> {
        self.conn
            .execute(
                "INSERT INTO categories (name, description) VALUES (?1, ?2)",
                params![name, description],
            )
            .map_err(|e| {
                // 捕获 UNIQUE 约束违反，返回友好的中文错误
                if let rusqlite::Error::SqliteFailure(ref err, _) = e {
                    if err.code == rusqlite::ErrorCode::ConstraintViolation {
                        return "分类名称已存在".to_string();
                    }
                }
                format!("创建分类失败：{}", e)
            })?;

        // 查询刚插入的记录并返回
        let id = self.conn.last_insert_rowid();
        self.conn
            .query_row(
                "SELECT id, name, description, created_at FROM categories WHERE id = ?1",
                params![id],
                |row| {
                    Ok(Category {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        description: row.get(2)?,
                        created_at: row.get(3)?,
                    })
                },
            )
            .map_err(|e| format!("创建分类失败：无法读取新记录: {}", e))
    }

    /// 查询所有分类
    ///
    /// # 返回
    /// - `Ok(Vec<Category>)`: 所有分类列表（按 id 升序）
    /// - `Err(String)`: 查询失败，返回中文错误描述
    pub fn list_categories(&self) -> Result<Vec<Category>, String> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, name, description, created_at FROM categories ORDER BY id")
            .map_err(|e| format!("查询分类失败：{}", e))?;

        let categories = stmt
            .query_map([], |row| {
                Ok(Category {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    created_at: row.get(3)?,
                })
            })
            .map_err(|e| format!("查询分类失败：{}", e))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("查询分类失败：读取记录时出错: {}", e))?;

        Ok(categories)
    }

    /// 更新分类
    ///
    /// # 参数
    /// - `id`: 分类 ID
    /// - `name`: 新的分类名称
    /// - `description`: 新的分类描述
    ///
    /// # 返回
    /// - `Ok(())`: 更新成功
    /// - `Err(String)`: 更新失败（如名称重复或 ID 不存在），返回中文错误描述
    pub fn update_category(
        &self,
        id: i64,
        name: &str,
        description: Option<&str>,
    ) -> Result<(), String> {
        let rows_affected = self
            .conn
            .execute(
                "UPDATE categories SET name = ?1, description = ?2 WHERE id = ?3",
                params![name, description, id],
            )
            .map_err(|e| {
                // 捕获 UNIQUE 约束违反
                if let rusqlite::Error::SqliteFailure(ref err, _) = e {
                    if err.code == rusqlite::ErrorCode::ConstraintViolation {
                        return "分类名称已存在".to_string();
                    }
                }
                format!("更新分类失败：{}", e)
            })?;

        if rows_affected == 0 {
            return Err(format!("更新分类失败：ID {} 不存在", id));
        }

        Ok(())
    }

    /// 删除分类
    ///
    /// 删除前检查是否有关联项目，如有则拒绝删除
    ///
    /// # 参数
    /// - `id`: 分类 ID
    ///
    /// # 返回
    /// - `Ok(())`: 删除成功
    /// - `Err(String)`: 删除失败（如有关联项目或 ID 不存在），返回中文错误描述
    pub fn delete_category(&self, id: i64) -> Result<(), String> {
        // 先查询该分类下的关联项目数
        let project_count: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM projects WHERE category_id = ?1",
                params![id],
                |row| row.get(0),
            )
            .map_err(|e| format!("删除分类失败：查询关联项目时出错: {}", e))?;

        // 如果有关联项目，拒绝删除
        if project_count > 0 {
            return Err("该分类下仍有项目，无法删除".to_string());
        }

        // 执行删除
        let rows_affected = self
            .conn
            .execute("DELETE FROM categories WHERE id = ?1", params![id])
            .map_err(|e| format!("删除分类失败：{}", e))?;

        if rows_affected == 0 {
            return Err(format!("删除分类失败：ID {} 不存在", id));
        }

        Ok(())
    }

    // ========================================================================
    // 项目 CRUD 方法
    // ========================================================================

    /// 创建项目
    ///
    /// 在插入前检查 repo_path 是否存在于文件系统，不存在则拒绝创建。
    ///
    /// # 参数
    /// - `name`: 项目名称
    /// - `category_id`: 所属分类 ID
    /// - `repo_path`: 仓库路径（必须在文件系统中存在）
    /// - `tech_stack`: 技术栈类型（如 "fastapi"、"vue3"）
    ///
    /// # 返回
    /// - `Ok(Project)`: 创建成功，返回完整的项目记录
    /// - `Err(String)`: 创建失败（如路径不存在），返回中文错误描述
    pub fn create_project(
        &self,
        name: &str,
        category_id: i64,
        repo_path: &str,
        tech_stack: &str,
    ) -> Result<Project, String> {
        // 检查仓库路径是否存在于文件系统
        if !std::path::Path::new(repo_path).exists() {
            return Err(format!("项目路径不存在：{}", repo_path));
        }

        // 插入项目记录
        self.conn
            .execute(
                "INSERT INTO projects (name, category_id, repo_path, tech_stack_type) VALUES (?1, ?2, ?3, ?4)",
                params![name, category_id, repo_path, tech_stack],
            )
            .map_err(|e| format!("创建项目失败：{}", e))?;

        // 查询刚插入的记录并返回
        let id = self.conn.last_insert_rowid();
        self.conn
            .query_row(
                "SELECT id, name, category_id, repo_path, tech_stack_type, created_at, updated_at FROM projects WHERE id = ?1",
                params![id],
                |row| {
                    Ok(Project {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        category_id: row.get(2)?,
                        repo_path: row.get(3)?,
                        tech_stack_type: row.get(4)?,
                        created_at: row.get(5)?,
                        updated_at: row.get(6)?,
                    })
                },
            )
            .map_err(|e| format!("创建项目失败：无法读取新记录: {}", e))
    }

    /// 查询所有项目
    ///
    /// # 返回
    /// - `Ok(Vec<Project>)`: 所有项目列表（按 id 升序）
    /// - `Err(String)`: 查询失败，返回中文错误描述
    pub fn list_projects(&self) -> Result<Vec<Project>, String> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, name, category_id, repo_path, tech_stack_type, created_at, updated_at FROM projects ORDER BY id")
            .map_err(|e| format!("查询项目失败：{}", e))?;

        let projects = stmt
            .query_map([], |row| {
                Ok(Project {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    category_id: row.get(2)?,
                    repo_path: row.get(3)?,
                    tech_stack_type: row.get(4)?,
                    created_at: row.get(5)?,
                    updated_at: row.get(6)?,
                })
            })
            .map_err(|e| format!("查询项目失败：{}", e))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("查询项目失败：读取记录时出错: {}", e))?;

        Ok(projects)
    }

    /// 根据 ID 查询单个项目
    ///
    /// # 参数
    /// - `id`: 项目 ID
    ///
    /// # 返回
    /// - `Ok(Project)`: 查询到的项目记录
    /// - `Err(String)`: 查询失败（如 ID 不存在），返回中文错误描述
    pub fn get_project(&self, id: i64) -> Result<Project, String> {
        self.conn
            .query_row(
                "SELECT id, name, category_id, repo_path, tech_stack_type, created_at, updated_at FROM projects WHERE id = ?1",
                params![id],
                |row| {
                    Ok(Project {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        category_id: row.get(2)?,
                        repo_path: row.get(3)?,
                        tech_stack_type: row.get(4)?,
                        created_at: row.get(5)?,
                        updated_at: row.get(6)?,
                    })
                },
            )
            .map_err(|e| {
                if matches!(e, rusqlite::Error::QueryReturnedNoRows) {
                    format!("查询项目失败：ID {} 不存在", id)
                } else {
                    format!("查询项目失败：{}", e)
                }
            })
    }

    /// 更新项目
    ///
    /// 更新项目的名称、分类和技术栈类型，同时更新 updated_at 时间戳。
    ///
    /// # 参数
    /// - `id`: 项目 ID
    /// - `name`: 新的项目名称
    /// - `category_id`: 新的分类 ID
    /// - `tech_stack`: 新的技术栈类型
    ///
    /// # 返回
    /// - `Ok(())`: 更新成功
    /// - `Err(String)`: 更新失败（如 ID 不存在），返回中文错误描述
    pub fn update_project(
        &self,
        id: i64,
        name: &str,
        category_id: i64,
        tech_stack: &str,
    ) -> Result<(), String> {
        let rows_affected = self
            .conn
            .execute(
                "UPDATE projects SET name = ?1, category_id = ?2, tech_stack_type = ?3, updated_at = datetime('now') WHERE id = ?4",
                params![name, category_id, tech_stack, id],
            )
            .map_err(|e| format!("更新项目失败：{}", e))?;

        if rows_affected == 0 {
            return Err(format!("更新项目失败：ID {} 不存在", id));
        }

        Ok(())
    }

    /// 删除项目
    ///
    /// 依赖 ON DELETE CASCADE 自动清理 project_clients 和 build_records 中的关联记录。
    ///
    /// # 参数
    /// - `id`: 项目 ID
    ///
    /// # 返回
    /// - `Ok(())`: 删除成功
    /// - `Err(String)`: 删除失败（如 ID 不存在），返回中文错误描述
    pub fn delete_project(&self, id: i64) -> Result<(), String> {
        let rows_affected = self
            .conn
            .execute("DELETE FROM projects WHERE id = ?1", params![id])
            .map_err(|e| format!("删除项目失败：{}", e))?;

        if rows_affected == 0 {
            return Err(format!("删除项目失败：ID {} 不存在", id));
        }

        Ok(())
    }

    // ========================================================================
    // 客户 CRUD 方法
    // ========================================================================

    /// 创建客户并关联到指定项目
    ///
    /// 在 clients 表中插入客户记录，然后在 project_clients 表中为每个
    /// project_id 创建关联记录。
    ///
    /// # 参数
    /// - `name`: 客户名称
    /// - `project_ids`: 要关联的项目 ID 列表
    ///
    /// # 返回
    /// - `Ok(Client)`: 创建成功，返回完整的客户记录
    /// - `Err(String)`: 创建失败，返回中文错误描述
    pub fn create_client(&self, name: &str, project_ids: &[i64]) -> Result<Client, String> {
        // 插入客户记录
        self.conn
            .execute("INSERT INTO clients (name) VALUES (?1)", params![name])
            .map_err(|e| format!("创建客户失败：{}", e))?;

        let client_id = self.conn.last_insert_rowid();

        // 为每个项目创建关联记录
        for &project_id in project_ids {
            self.conn
                .execute(
                    "INSERT INTO project_clients (project_id, client_id) VALUES (?1, ?2)",
                    params![project_id, client_id],
                )
                .map_err(|e| format!("创建客户关联失败：{}", e))?;
        }

        // 查询刚插入的客户记录并返回
        self.conn
            .query_row(
                "SELECT id, name, created_at FROM clients WHERE id = ?1",
                params![client_id],
                |row| {
                    Ok(Client {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        created_at: row.get(2)?,
                    })
                },
            )
            .map_err(|e| format!("创建客户失败：无法读取新记录: {}", e))
    }

    /// 查询指定项目关联的所有客户
    ///
    /// 通过 JOIN project_clients 表过滤，仅返回与指定项目关联的客户。
    ///
    /// # 参数
    /// - `project_id`: 项目 ID
    ///
    /// # 返回
    /// - `Ok(Vec<Client>)`: 关联客户列表（按 id 升序）
    /// - `Err(String)`: 查询失败，返回中文错误描述
    pub fn list_clients_by_project(&self, project_id: i64) -> Result<Vec<Client>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT c.id, c.name, c.created_at
                 FROM clients c
                 INNER JOIN project_clients pc ON c.id = pc.client_id
                 WHERE pc.project_id = ?1
                 ORDER BY c.id",
            )
            .map_err(|e| format!("查询客户失败：{}", e))?;

        let clients = stmt
            .query_map(params![project_id], |row| {
                Ok(Client {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    created_at: row.get(2)?,
                })
            })
            .map_err(|e| format!("查询客户失败：{}", e))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("查询客户失败：读取记录时出错: {}", e))?;

        Ok(clients)
    }

    /// 更新客户名称
    ///
    /// # 参数
    /// - `id`: 客户 ID
    /// - `name`: 新的客户名称
    ///
    /// # 返回
    /// - `Ok(())`: 更新成功
    /// - `Err(String)`: 更新失败（如 ID 不存在），返回中文错误描述
    pub fn update_client(&self, id: i64, name: &str) -> Result<(), String> {
        let rows_affected = self
            .conn
            .execute(
                "UPDATE clients SET name = ?1 WHERE id = ?2",
                params![name, id],
            )
            .map_err(|e| format!("更新客户失败：{}", e))?;

        if rows_affected == 0 {
            return Err(format!("更新客户失败：ID {} 不存在", id));
        }

        Ok(())
    }

    /// 删除客户
    ///
    /// 依赖 ON DELETE CASCADE 自动清理 project_clients 中的关联记录。
    ///
    /// # 参数
    /// - `id`: 客户 ID
    ///
    /// # 返回
    /// - `Ok(())`: 删除成功
    /// - `Err(String)`: 删除失败（如 ID 不存在），返回中文错误描述
    pub fn delete_client(&self, id: i64) -> Result<(), String> {
        let rows_affected = self
            .conn
            .execute("DELETE FROM clients WHERE id = ?1", params![id])
            .map_err(|e| format!("删除客户失败：{}", e))?;

        if rows_affected == 0 {
            return Err(format!("删除客户失败：ID {} 不存在", id));
        }

        Ok(())
    }

    // ========================================================================
    // 构建记录方法
    // ========================================================================

    /// 创建构建记录
    ///
    /// 将一次构建操作的信息持久化到 build_records 表中。
    /// selected_modules 以 JSON 字符串形式存储。
    ///
    /// # 参数
    /// - `project_id`: 关联的项目 ID
    /// - `client_id`: 关联的客户 ID
    /// - `modules_json`: 选中模块的 JSON 数组字符串
    /// - `output_path`: 构建输出文件路径
    ///
    /// # 返回
    /// - `Ok(BuildRecord)`: 创建成功，返回完整的构建记录
    /// - `Err(String)`: 创建失败，返回中文错误描述
    pub fn create_build_record(
        &self,
        project_id: i64,
        client_id: i64,
        modules_json: &str,
        output_path: &str,
    ) -> Result<BuildRecord, String> {
        self.conn
            .execute(
                "INSERT INTO build_records (project_id, client_id, selected_modules, output_path) VALUES (?1, ?2, ?3, ?4)",
                params![project_id, client_id, modules_json, output_path],
            )
            .map_err(|e| format!("创建构建记录失败：{}", e))?;

        let id = self.conn.last_insert_rowid();

        // 查询刚插入的记录以获取完整字段（包括 created_at 默认值）
        self.conn
            .query_row(
                "SELECT id, project_id, client_id, selected_modules, output_path, created_at FROM build_records WHERE id = ?1",
                params![id],
                |row| {
                    Ok(BuildRecord {
                        id: row.get(0)?,
                        project_id: row.get(1)?,
                        client_id: row.get(2)?,
                        selected_modules: row.get(3)?,
                        output_path: row.get(4)?,
                        created_at: row.get(5)?,
                    })
                },
            )
            .map_err(|e| format!("查询构建记录失败：{}", e))
    }

    /// 按项目 ID 查询构建记录列表
    ///
    /// 返回指定项目的所有构建记录，按创建时间倒序排列（最新的在前）。
    ///
    /// # 参数
    /// - `project_id`: 项目 ID
    ///
    /// # 返回
    /// - `Ok(Vec<BuildRecord>)`: 查询成功，返回构建记录列表
    /// - `Err(String)`: 查询失败，返回中文错误描述
    pub fn list_build_records_by_project(
        &self,
        project_id: i64,
    ) -> Result<Vec<BuildRecord>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, project_id, client_id, selected_modules, output_path, created_at FROM build_records WHERE project_id = ?1 ORDER BY created_at DESC, id DESC",
            )
            .map_err(|e| format!("查询构建记录失败：{}", e))?;

        let records = stmt
            .query_map(params![project_id], |row| {
                Ok(BuildRecord {
                    id: row.get(0)?,
                    project_id: row.get(1)?,
                    client_id: row.get(2)?,
                    selected_modules: row.get(3)?,
                    output_path: row.get(4)?,
                    created_at: row.get(5)?,
                })
            })
            .map_err(|e| format!("查询构建记录失败：{}", e))?;

        records
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("读取构建记录失败：{}", e))
    }

    // ========================================================================
    // 设置方法（键值对操作）
    // ========================================================================

    /// 获取应用设置
    ///
    /// 从 settings 表中读取所有设置项，构造 AppSettings 结构体。
    /// 当前支持的设置键：
    /// - "default_output_dir": 默认构建输出目录
    ///
    /// # 参数
    /// - `db_path`: 数据库文件路径（直接传入，不从数据库读取）
    ///
    /// # 返回
    /// - `Ok(AppSettings)`: 查询成功，返回应用设置
    /// - `Err(String)`: 查询失败，返回中文错误描述
    pub fn get_settings(&self, db_path: &str) -> Result<AppSettings, String> {
        // 查询 default_output_dir 设置项
        let default_output_dir: Option<String> = self
            .conn
            .query_row(
                "SELECT value FROM settings WHERE key = ?1",
                params!["default_output_dir"],
                |row| row.get(0),
            )
            .ok(); // 如果键不存在，返回 None

        Ok(AppSettings {
            default_output_dir,
            db_path: db_path.to_string(),
        })
    }

    /// 保存单个设置项（键值对）
    ///
    /// 使用 INSERT OR REPLACE 实现 upsert 语义：
    /// - 如果键不存在，插入新记录
    /// - 如果键已存在，更新其值
    ///
    /// # 参数
    /// - `key`: 设置键名
    /// - `value`: 设置值
    ///
    /// # 返回
    /// - `Ok(())`: 保存成功
    /// - `Err(String)`: 保存失败，返回中文错误描述
    pub fn save_setting(&self, key: &str, value: &str) -> Result<(), String> {
        self.conn
            .execute(
                "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
                params![key, value],
            )
            .map_err(|e| format!("保存设置失败：{}", e))?;

        Ok(())
    }
}

// ============================================================================
// 单元测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use rusqlite::params;
    use tempfile::TempDir;

    /// 测试数据库初始化：创建文件和所有表
    #[test]
    fn test_database_init_creates_file_and_tables() {
        let dir = TempDir::new().unwrap();
        let db = Database::init(dir.path()).unwrap();

        // 验证数据库文件已创建
        assert!(dir.path().join("prism_console.db").exists());

        // 验证六张表都已创建（通过查询 sqlite_master）
        let table_names: Vec<String> = db
            .conn()
            .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();

        assert_eq!(table_names.len(), 6);
        assert!(table_names.contains(&"categories".to_string()));
        assert!(table_names.contains(&"projects".to_string()));
        assert!(table_names.contains(&"clients".to_string()));
        assert!(table_names.contains(&"project_clients".to_string()));
        assert!(table_names.contains(&"build_records".to_string()));
        assert!(table_names.contains(&"settings".to_string()));
    }

    /// 测试数据库初始化：外键约束已启用
    #[test]
    fn test_database_init_foreign_keys_enabled() {
        let dir = TempDir::new().unwrap();
        let db = Database::init(dir.path()).unwrap();

        // 验证外键约束已启用
        let fk_enabled: i32 = db
            .conn()
            .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
            .unwrap();
        assert_eq!(fk_enabled, 1);
    }

    /// 测试数据库初始化：重复初始化不会报错（CREATE TABLE IF NOT EXISTS）
    #[test]
    fn test_database_init_idempotent() {
        let dir = TempDir::new().unwrap();

        // 第一次初始化
        let _db1 = Database::init(dir.path()).unwrap();
        // 第二次初始化（同一目录），不应报错
        let db2 = Database::init(dir.path()).unwrap();

        // 验证表仍然存在
        let count: i32 = db2
            .conn()
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 6);
    }

    /// 测试数据库初始化：自动创建不存在的目录
    #[test]
    fn test_database_init_creates_directory() {
        let dir = TempDir::new().unwrap();
        let nested_path = dir.path().join("nested").join("deep").join("data");

        let db = Database::init(&nested_path).unwrap();

        // 验证嵌套目录和数据库文件都已创建
        assert!(nested_path.join("prism_console.db").exists());

        // 验证表已创建
        let count: i32 = db
            .conn()
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 6);
    }

    /// 测试 categories 表结构：验证列定义
    #[test]
    fn test_categories_table_schema() {
        let dir = TempDir::new().unwrap();
        let db = Database::init(dir.path()).unwrap();

        // 插入一条分类记录验证表结构
        db.conn()
            .execute(
                "INSERT INTO categories (name, description) VALUES (?1, ?2)",
                params!["测试分类", "这是一个测试分类"],
            )
            .unwrap();

        // 查询验证
        let (id, name, desc, created_at): (i64, String, Option<String>, String) = db
            .conn()
            .query_row(
                "SELECT id, name, description, created_at FROM categories WHERE name = ?1",
                params!["测试分类"],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .unwrap();

        assert!(id > 0);
        assert_eq!(name, "测试分类");
        assert_eq!(desc, Some("这是一个测试分类".to_string()));
        assert!(!created_at.is_empty());
    }

    /// 测试 categories 表的 UNIQUE 约束
    #[test]
    fn test_categories_unique_name_constraint() {
        let dir = TempDir::new().unwrap();
        let db = Database::init(dir.path()).unwrap();

        // 第一次插入成功
        db.conn()
            .execute(
                "INSERT INTO categories (name) VALUES (?1)",
                params!["唯一分类"],
            )
            .unwrap();

        // 第二次插入相同名称应失败
        let result = db.conn().execute(
            "INSERT INTO categories (name) VALUES (?1)",
            params!["唯一分类"],
        );
        assert!(result.is_err());
    }

    /// 测试 projects 表结构：验证外键关联
    #[test]
    fn test_projects_table_with_foreign_key() {
        let dir = TempDir::new().unwrap();
        let db = Database::init(dir.path()).unwrap();

        // 先创建分类
        db.conn()
            .execute("INSERT INTO categories (name) VALUES (?1)", params!["后端"])
            .unwrap();
        let category_id: i64 = db
            .conn()
            .query_row("SELECT last_insert_rowid()", [], |row| row.get(0))
            .unwrap();

        // 创建项目
        db.conn()
            .execute(
                "INSERT INTO projects (name, category_id, repo_path, tech_stack_type) VALUES (?1, ?2, ?3, ?4)",
                params!["测试项目", category_id, "/path/to/repo", "fastapi"],
            )
            .unwrap();

        // 查询验证
        let (name, tech_stack): (String, String) = db
            .conn()
            .query_row(
                "SELECT name, tech_stack_type FROM projects WHERE category_id = ?1",
                params![category_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();

        assert_eq!(name, "测试项目");
        assert_eq!(tech_stack, "fastapi");
    }

    /// 测试 project_clients 关联表：多对多关系
    #[test]
    fn test_project_clients_many_to_many() {
        let dir = TempDir::new().unwrap();
        let db = Database::init(dir.path()).unwrap();

        // 创建分类
        db.conn()
            .execute("INSERT INTO categories (name) VALUES (?1)", params!["分类"])
            .unwrap();

        // 创建项目
        db.conn()
            .execute(
                "INSERT INTO projects (name, category_id, repo_path) VALUES (?1, 1, ?2)",
                params!["项目A", "/path/a"],
            )
            .unwrap();
        db.conn()
            .execute(
                "INSERT INTO projects (name, category_id, repo_path) VALUES (?1, 1, ?2)",
                params!["项目B", "/path/b"],
            )
            .unwrap();

        // 创建客户
        db.conn()
            .execute("INSERT INTO clients (name) VALUES (?1)", params!["客户X"])
            .unwrap();

        // 建立关联：客户X 关联到 项目A 和 项目B
        db.conn()
            .execute(
                "INSERT INTO project_clients (project_id, client_id) VALUES (?1, ?2)",
                params![1, 1],
            )
            .unwrap();
        db.conn()
            .execute(
                "INSERT INTO project_clients (project_id, client_id) VALUES (?1, ?2)",
                params![2, 1],
            )
            .unwrap();

        // 查询客户X关联的项目数
        let count: i32 = db
            .conn()
            .query_row(
                "SELECT COUNT(*) FROM project_clients WHERE client_id = ?1",
                params![1],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 2);
    }

    /// 测试 ON DELETE CASCADE：删除项目时自动清理关联数据
    #[test]
    fn test_cascade_delete_project() {
        let dir = TempDir::new().unwrap();
        let db = Database::init(dir.path()).unwrap();

        // 创建分类 -> 项目 -> 客户 -> 关联 -> 构建记录
        db.conn()
            .execute("INSERT INTO categories (name) VALUES (?1)", params!["分类"])
            .unwrap();
        db.conn()
            .execute(
                "INSERT INTO projects (name, category_id, repo_path) VALUES (?1, 1, ?2)",
                params!["项目", "/path"],
            )
            .unwrap();
        db.conn()
            .execute("INSERT INTO clients (name) VALUES (?1)", params!["客户"])
            .unwrap();
        db.conn()
            .execute(
                "INSERT INTO project_clients (project_id, client_id) VALUES (1, 1)",
                [],
            )
            .unwrap();
        db.conn()
            .execute(
                "INSERT INTO build_records (project_id, client_id, selected_modules, output_path) VALUES (1, 1, ?1, ?2)",
                params!["[\"auth\"]", "/output/path"],
            )
            .unwrap();

        // 删除项目
        db.conn()
            .execute("DELETE FROM projects WHERE id = 1", [])
            .unwrap();

        // 验证级联删除：project_clients 和 build_records 中的关联记录应被清除
        let pc_count: i32 = db
            .conn()
            .query_row(
                "SELECT COUNT(*) FROM project_clients WHERE project_id = 1",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(pc_count, 0);

        let br_count: i32 = db
            .conn()
            .query_row(
                "SELECT COUNT(*) FROM build_records WHERE project_id = 1",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(br_count, 0);

        // 客户本身不应被删除
        let client_count: i32 = db
            .conn()
            .query_row("SELECT COUNT(*) FROM clients", [], |row| row.get(0))
            .unwrap();
        assert_eq!(client_count, 1);
    }

    // ========================================================================
    // Category CRUD 方法单元测试
    // ========================================================================

    /// 测试 create_category：正常创建分类
    #[test]
    fn test_create_category_success() {
        let dir = TempDir::new().unwrap();
        let db = Database::init(dir.path()).unwrap();

        let cat = db.create_category("前端", Some("前端项目分类")).unwrap();
        assert!(cat.id > 0);
        assert_eq!(cat.name, "前端");
        assert_eq!(cat.description, Some("前端项目分类".to_string()));
        assert!(!cat.created_at.is_empty());
    }

    /// 测试 create_category：无描述创建
    #[test]
    fn test_create_category_without_description() {
        let dir = TempDir::new().unwrap();
        let db = Database::init(dir.path()).unwrap();

        let cat = db.create_category("后端", None).unwrap();
        assert_eq!(cat.name, "后端");
        assert_eq!(cat.description, None);
    }

    /// 测试 create_category：重复名称返回中文错误
    #[test]
    fn test_create_category_duplicate_name() {
        let dir = TempDir::new().unwrap();
        let db = Database::init(dir.path()).unwrap();

        db.create_category("工具类", None).unwrap();
        let err = db.create_category("工具类", None).unwrap_err();
        assert_eq!(err, "分类名称已存在");
    }

    /// 测试 list_categories：列出所有分类
    #[test]
    fn test_list_categories() {
        let dir = TempDir::new().unwrap();
        let db = Database::init(dir.path()).unwrap();

        // 空列表
        let cats = db.list_categories().unwrap();
        assert!(cats.is_empty());

        // 创建两个分类后列出
        db.create_category("前端", None).unwrap();
        db.create_category("后端", Some("后端服务")).unwrap();

        let cats = db.list_categories().unwrap();
        assert_eq!(cats.len(), 2);
        assert_eq!(cats[0].name, "前端");
        assert_eq!(cats[1].name, "后端");
        assert_eq!(cats[1].description, Some("后端服务".to_string()));
    }

    /// 测试 update_category：正常更新
    #[test]
    fn test_update_category_success() {
        let dir = TempDir::new().unwrap();
        let db = Database::init(dir.path()).unwrap();

        let cat = db.create_category("旧名称", None).unwrap();
        db.update_category(cat.id, "新名称", Some("新描述"))
            .unwrap();

        let cats = db.list_categories().unwrap();
        assert_eq!(cats.len(), 1);
        assert_eq!(cats[0].name, "新名称");
        assert_eq!(cats[0].description, Some("新描述".to_string()));
    }

    /// 测试 update_category：更新为已存在的名称
    #[test]
    fn test_update_category_duplicate_name() {
        let dir = TempDir::new().unwrap();
        let db = Database::init(dir.path()).unwrap();

        db.create_category("分类A", None).unwrap();
        let cat_b = db.create_category("分类B", None).unwrap();

        let err = db.update_category(cat_b.id, "分类A", None).unwrap_err();
        assert_eq!(err, "分类名称已存在");
    }

    /// 测试 update_category：不存在的 ID
    #[test]
    fn test_update_category_not_found() {
        let dir = TempDir::new().unwrap();
        let db = Database::init(dir.path()).unwrap();

        let err = db.update_category(999, "不存在", None).unwrap_err();
        assert!(err.contains("不存在"));
    }

    /// 测试 delete_category：正常删除
    #[test]
    fn test_delete_category_success() {
        let dir = TempDir::new().unwrap();
        let db = Database::init(dir.path()).unwrap();

        let cat = db.create_category("待删除", None).unwrap();
        db.delete_category(cat.id).unwrap();

        let cats = db.list_categories().unwrap();
        assert!(cats.is_empty());
    }

    /// 测试 delete_category：有关联项目时拒绝删除
    #[test]
    fn test_delete_category_with_projects() {
        let dir = TempDir::new().unwrap();
        let db = Database::init(dir.path()).unwrap();

        let cat = db.create_category("有项目的分类", None).unwrap();

        // 手动插入一个关联项目
        db.conn()
            .execute(
                "INSERT INTO projects (name, category_id, repo_path, tech_stack_type) VALUES (?1, ?2, ?3, ?4)",
                params!["测试项目", cat.id, "/path/to/repo", "fastapi"],
            )
            .unwrap();

        let err = db.delete_category(cat.id).unwrap_err();
        assert_eq!(err, "该分类下仍有项目，无法删除");

        // 验证分类仍然存在
        let cats = db.list_categories().unwrap();
        assert_eq!(cats.len(), 1);
    }

    /// 测试 delete_category：不存在的 ID
    #[test]
    fn test_delete_category_not_found() {
        let dir = TempDir::new().unwrap();
        let db = Database::init(dir.path()).unwrap();

        let err = db.delete_category(999).unwrap_err();
        assert!(err.contains("不存在"));
    }

    /// 测试 settings 表：键值对存储
    #[test]
    fn test_settings_key_value_store() {
        let dir = TempDir::new().unwrap();
        let db = Database::init(dir.path()).unwrap();

        // 插入设置
        db.conn()
            .execute(
                "INSERT INTO settings (key, value) VALUES (?1, ?2)",
                params!["default_output_dir", "/home/user/output"],
            )
            .unwrap();

        // 查询设置
        let value: String = db
            .conn()
            .query_row(
                "SELECT value FROM settings WHERE key = ?1",
                params!["default_output_dir"],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(value, "/home/user/output");

        // 更新设置（使用 INSERT OR REPLACE）
        db.conn()
            .execute(
                "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
                params!["default_output_dir", "/new/path"],
            )
            .unwrap();

        let updated_value: String = db
            .conn()
            .query_row(
                "SELECT value FROM settings WHERE key = ?1",
                params!["default_output_dir"],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(updated_value, "/new/path");
    }

    // ========================================================================
    // Build Record 方法单元测试
    // ========================================================================

    /// 辅助函数：创建测试用的项目和客户，返回 (Database, project_id, client_id)
    fn setup_project_and_client() -> (Database, TempDir, i64, i64) {
        let dir = TempDir::new().unwrap();
        let db = Database::init(dir.path()).unwrap();

        // 创建分类
        let cat = db.create_category("测试分类", None).unwrap();

        // 创建项目（使用临时目录作为仓库路径）
        let repo_dir = TempDir::new().unwrap();
        let repo_path = repo_dir.path().to_str().unwrap().to_string();
        let project = db
            .create_project("测试项目", cat.id, &repo_path, "fastapi")
            .unwrap();

        // 创建客户并关联到项目
        let client = db.create_client("测试客户", &[project.id]).unwrap();

        // 需要保持 repo_dir 存活，但这里我们把 dir 返回出去
        // repo_dir 在函数结束后会被 drop，但项目已经创建成功了
        (db, dir, project.id, client.id)
    }

    /// 测试 create_build_record：正常创建构建记录
    #[test]
    fn test_create_build_record_success() {
        let (db, _dir, project_id, client_id) = setup_project_and_client();

        let modules_json = r#"["module_a","module_b"]"#;
        let output_path = "/tmp/output/test.zip";

        let record = db
            .create_build_record(project_id, client_id, modules_json, output_path)
            .unwrap();

        assert!(record.id > 0);
        assert_eq!(record.project_id, project_id);
        assert_eq!(record.client_id, client_id);
        assert_eq!(record.selected_modules, modules_json);
        assert_eq!(record.output_path, output_path);
        assert!(!record.created_at.is_empty());
    }

    /// 测试 create_build_record：selected_modules 以 JSON 字符串存储
    #[test]
    fn test_create_build_record_json_modules() {
        let (db, _dir, project_id, client_id) = setup_project_and_client();

        let modules_json = r#"["auth","users","orders"]"#;
        let record = db
            .create_build_record(project_id, client_id, modules_json, "/tmp/out.zip")
            .unwrap();

        // 验证 JSON 字符串原样存储和读取
        assert_eq!(record.selected_modules, modules_json);
    }

    /// 测试 list_build_records_by_project：按项目查询并按时间倒序
    #[test]
    fn test_list_build_records_by_project() {
        let (db, _dir, project_id, client_id) = setup_project_and_client();

        // 创建多条构建记录
        let r1 = db
            .create_build_record(project_id, client_id, r#"["mod_a"]"#, "/tmp/out1.zip")
            .unwrap();
        let r2 = db
            .create_build_record(project_id, client_id, r#"["mod_b"]"#, "/tmp/out2.zip")
            .unwrap();

        let records = db.list_build_records_by_project(project_id).unwrap();
        assert_eq!(records.len(), 2);

        // 按 created_at DESC 排序，最新的在前
        // 由于 SQLite datetime('now') 精度可能相同，用 id 辅助验证顺序
        assert_eq!(records[0].id, r2.id);
        assert_eq!(records[1].id, r1.id);
    }

    /// 测试 list_build_records_by_project：空结果
    #[test]
    fn test_list_build_records_by_project_empty() {
        let (db, _dir, project_id, _client_id) = setup_project_and_client();

        let records = db.list_build_records_by_project(project_id).unwrap();
        assert!(records.is_empty());
    }

    /// 测试 list_build_records_by_project：不同项目的记录互不干扰
    #[test]
    fn test_list_build_records_by_project_isolation() {
        let dir = TempDir::new().unwrap();
        let db = Database::init(dir.path()).unwrap();

        let cat = db.create_category("分类A", None).unwrap();

        // 创建两个项目
        let repo_dir_a = TempDir::new().unwrap();
        let repo_dir_b = TempDir::new().unwrap();
        let project_a = db
            .create_project(
                "项目A",
                cat.id,
                repo_dir_a.path().to_str().unwrap(),
                "fastapi",
            )
            .unwrap();
        let project_b = db
            .create_project("项目B", cat.id, repo_dir_b.path().to_str().unwrap(), "vue3")
            .unwrap();

        // 创建客户
        let client = db
            .create_client("客户X", &[project_a.id, project_b.id])
            .unwrap();

        // 为项目 A 创建 2 条记录
        db.create_build_record(project_a.id, client.id, r#"["a1"]"#, "/tmp/a1.zip")
            .unwrap();
        db.create_build_record(project_a.id, client.id, r#"["a2"]"#, "/tmp/a2.zip")
            .unwrap();

        // 为项目 B 创建 1 条记录
        db.create_build_record(project_b.id, client.id, r#"["b1"]"#, "/tmp/b1.zip")
            .unwrap();

        // 查询项目 A 的记录
        let records_a = db.list_build_records_by_project(project_a.id).unwrap();
        assert_eq!(records_a.len(), 2);
        assert!(records_a.iter().all(|r| r.project_id == project_a.id));

        // 查询项目 B 的记录
        let records_b = db.list_build_records_by_project(project_b.id).unwrap();
        assert_eq!(records_b.len(), 1);
        assert_eq!(records_b[0].project_id, project_b.id);
    }

    // ========================================================================
    // Settings 方法单元测试
    // ========================================================================

    /// 测试 get_settings：无设置时返回默认值
    #[test]
    fn test_get_settings_default() {
        let dir = TempDir::new().unwrap();
        let db = Database::init(dir.path()).unwrap();

        let settings = db.get_settings("/path/to/db").unwrap();
        assert_eq!(settings.default_output_dir, None);
        assert_eq!(settings.db_path, "/path/to/db");
    }

    /// 测试 save_setting + get_settings：保存后读取
    #[test]
    fn test_save_and_get_settings() {
        let dir = TempDir::new().unwrap();
        let db = Database::init(dir.path()).unwrap();

        // 保存设置
        db.save_setting("default_output_dir", "/home/user/output")
            .unwrap();

        // 读取设置
        let settings = db.get_settings("/path/to/db").unwrap();
        assert_eq!(
            settings.default_output_dir,
            Some("/home/user/output".to_string())
        );
        assert_eq!(settings.db_path, "/path/to/db");
    }

    /// 测试 save_setting：更新已有设置（upsert 语义）
    #[test]
    fn test_save_setting_upsert() {
        let dir = TempDir::new().unwrap();
        let db = Database::init(dir.path()).unwrap();

        // 首次保存
        db.save_setting("default_output_dir", "/old/path").unwrap();
        let settings = db.get_settings("/db").unwrap();
        assert_eq!(settings.default_output_dir, Some("/old/path".to_string()));

        // 更新同一个键
        db.save_setting("default_output_dir", "/new/path").unwrap();
        let settings = db.get_settings("/db").unwrap();
        assert_eq!(settings.default_output_dir, Some("/new/path".to_string()));
    }

    /// 测试 save_setting：保存多个不同的键
    #[test]
    fn test_save_setting_multiple_keys() {
        let dir = TempDir::new().unwrap();
        let db = Database::init(dir.path()).unwrap();

        db.save_setting("default_output_dir", "/output").unwrap();
        db.save_setting("theme", "dark").unwrap();

        // get_settings 只读取 default_output_dir
        let settings = db.get_settings("/db").unwrap();
        assert_eq!(settings.default_output_dir, Some("/output".to_string()));

        // 验证其他键也确实存储了
        let theme: String = db
            .conn()
            .query_row(
                "SELECT value FROM settings WHERE key = ?1",
                params!["theme"],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(theme, "dark");
    }

    // ========================================================================
    // Project CRUD 方法单元测试
    // ========================================================================

    /// 测试 create_project：正常创建项目（使用真实存在的路径）
    #[test]
    fn test_create_project_success() {
        let dir = TempDir::new().unwrap();
        let db = Database::init(dir.path()).unwrap();

        // 先创建分类
        let cat = db.create_category("后端", None).unwrap();

        // 使用临时目录作为仓库路径（真实存在的路径）
        let repo_dir = TempDir::new().unwrap();
        let repo_path = repo_dir.path().to_str().unwrap();

        let project = db
            .create_project("测试项目", cat.id, repo_path, "fastapi")
            .unwrap();
        assert!(project.id > 0);
        assert_eq!(project.name, "测试项目");
        assert_eq!(project.category_id, cat.id);
        assert_eq!(project.repo_path, repo_path);
        assert_eq!(project.tech_stack_type, "fastapi");
        assert!(!project.created_at.is_empty());
        assert!(!project.updated_at.is_empty());
    }

    /// 测试 create_project：仓库路径不存在时返回中文错误
    #[test]
    fn test_create_project_path_not_exists() {
        let dir = TempDir::new().unwrap();
        let db = Database::init(dir.path()).unwrap();

        let cat = db.create_category("前端", None).unwrap();

        let fake_path = "/this/path/does/not/exist/at/all";
        let err = db
            .create_project("项目X", cat.id, fake_path, "vue3")
            .unwrap_err();
        assert_eq!(err, format!("项目路径不存在：{}", fake_path));
    }

    /// 测试 list_projects：列出所有项目
    #[test]
    fn test_list_projects() {
        let dir = TempDir::new().unwrap();
        let db = Database::init(dir.path()).unwrap();

        // 空列表
        let projects = db.list_projects().unwrap();
        assert!(projects.is_empty());

        // 创建分类和项目
        let cat = db.create_category("分类", None).unwrap();
        let repo1 = TempDir::new().unwrap();
        let repo2 = TempDir::new().unwrap();

        db.create_project("项目A", cat.id, repo1.path().to_str().unwrap(), "fastapi")
            .unwrap();
        db.create_project("项目B", cat.id, repo2.path().to_str().unwrap(), "vue3")
            .unwrap();

        let projects = db.list_projects().unwrap();
        assert_eq!(projects.len(), 2);
        assert_eq!(projects[0].name, "项目A");
        assert_eq!(projects[1].name, "项目B");
        assert_eq!(projects[0].tech_stack_type, "fastapi");
        assert_eq!(projects[1].tech_stack_type, "vue3");
    }

    /// 测试 get_project：根据 ID 查询项目
    #[test]
    fn test_get_project_success() {
        let dir = TempDir::new().unwrap();
        let db = Database::init(dir.path()).unwrap();

        let cat = db.create_category("分类", None).unwrap();
        let repo = TempDir::new().unwrap();
        let repo_path = repo.path().to_str().unwrap();

        let created = db
            .create_project("我的项目", cat.id, repo_path, "fastapi")
            .unwrap();
        let fetched = db.get_project(created.id).unwrap();

        assert_eq!(fetched.id, created.id);
        assert_eq!(fetched.name, "我的项目");
        assert_eq!(fetched.category_id, cat.id);
        assert_eq!(fetched.repo_path, repo_path);
        assert_eq!(fetched.tech_stack_type, "fastapi");
    }

    /// 测试 get_project：不存在的 ID
    #[test]
    fn test_get_project_not_found() {
        let dir = TempDir::new().unwrap();
        let db = Database::init(dir.path()).unwrap();

        let err = db.get_project(999).unwrap_err();
        assert!(err.contains("不存在"));
    }

    /// 测试 update_project：正常更新
    #[test]
    fn test_update_project_success() {
        let dir = TempDir::new().unwrap();
        let db = Database::init(dir.path()).unwrap();

        let cat1 = db.create_category("前端", None).unwrap();
        let cat2 = db.create_category("后端", None).unwrap();
        let repo = TempDir::new().unwrap();

        let project = db
            .create_project("旧名称", cat1.id, repo.path().to_str().unwrap(), "vue3")
            .unwrap();

        // 更新项目
        db.update_project(project.id, "新名称", cat2.id, "fastapi")
            .unwrap();

        // 验证更新结果
        let updated = db.get_project(project.id).unwrap();
        assert_eq!(updated.name, "新名称");
        assert_eq!(updated.category_id, cat2.id);
        assert_eq!(updated.tech_stack_type, "fastapi");
    }

    /// 测试 update_project：不存在的 ID
    #[test]
    fn test_update_project_not_found() {
        let dir = TempDir::new().unwrap();
        let db = Database::init(dir.path()).unwrap();

        let err = db.update_project(999, "名称", 1, "fastapi").unwrap_err();
        assert!(err.contains("不存在"));
    }

    /// 测试 delete_project：正常删除
    #[test]
    fn test_delete_project_success() {
        let dir = TempDir::new().unwrap();
        let db = Database::init(dir.path()).unwrap();

        let cat = db.create_category("分类", None).unwrap();
        let repo = TempDir::new().unwrap();

        let project = db
            .create_project("待删除", cat.id, repo.path().to_str().unwrap(), "fastapi")
            .unwrap();
        db.delete_project(project.id).unwrap();

        // 验证项目已被删除
        let projects = db.list_projects().unwrap();
        assert!(projects.is_empty());
    }

    /// 测试 delete_project：不存在的 ID
    #[test]
    fn test_delete_project_not_found() {
        let dir = TempDir::new().unwrap();
        let db = Database::init(dir.path()).unwrap();

        let err = db.delete_project(999).unwrap_err();
        assert!(err.contains("不存在"));
    }

    /// 测试 delete_project：级联删除 project_clients 和 build_records
    #[test]
    fn test_delete_project_cascade() {
        let dir = TempDir::new().unwrap();
        let db = Database::init(dir.path()).unwrap();

        // 创建分类 -> 项目 -> 客户 -> 关联 -> 构建记录
        let cat = db.create_category("分类", None).unwrap();
        let repo = TempDir::new().unwrap();
        let project = db
            .create_project("项目", cat.id, repo.path().to_str().unwrap(), "fastapi")
            .unwrap();

        // 手动插入客户和关联数据
        db.conn()
            .execute("INSERT INTO clients (name) VALUES (?1)", params!["客户A"])
            .unwrap();
        let client_id: i64 = db.conn().last_insert_rowid();

        db.conn()
            .execute(
                "INSERT INTO project_clients (project_id, client_id) VALUES (?1, ?2)",
                params![project.id, client_id],
            )
            .unwrap();
        db.conn()
            .execute(
                "INSERT INTO build_records (project_id, client_id, selected_modules, output_path) VALUES (?1, ?2, ?3, ?4)",
                params![project.id, client_id, "[\"auth\"]", "/output"],
            )
            .unwrap();

        // 删除项目
        db.delete_project(project.id).unwrap();

        // 验证级联删除：project_clients 中的关联记录应被清除
        let pc_count: i64 = db
            .conn()
            .query_row(
                "SELECT COUNT(*) FROM project_clients WHERE project_id = ?1",
                params![project.id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(pc_count, 0);

        // 验证级联删除：build_records 中的关联记录应被清除
        let br_count: i64 = db
            .conn()
            .query_row(
                "SELECT COUNT(*) FROM build_records WHERE project_id = ?1",
                params![project.id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(br_count, 0);

        // 客户本身不应被删除
        let client_count: i64 = db
            .conn()
            .query_row("SELECT COUNT(*) FROM clients", [], |row| row.get(0))
            .unwrap();
        assert_eq!(client_count, 1);
    }

    // ========================================================================
    // Client CRUD 单元测试
    // ========================================================================

    /// 测试 create_client：正常创建并关联项目
    #[test]
    fn test_create_client_success() {
        let dir = TempDir::new().unwrap();
        let db = Database::init(dir.path()).unwrap();

        // 创建分类和项目（用于关联）
        let cat = db.create_category("分类", None).unwrap();
        let repo = TempDir::new().unwrap();
        let project = db
            .create_project("项目A", cat.id, repo.path().to_str().unwrap(), "fastapi")
            .unwrap();

        // 创建客户并关联到项目
        let client = db.create_client("客户X", &[project.id]).unwrap();
        assert_eq!(client.name, "客户X");
        assert!(client.id > 0);

        // 验证 project_clients 关联记录已创建
        let pc_count: i64 = db
            .conn()
            .query_row(
                "SELECT COUNT(*) FROM project_clients WHERE client_id = ?1 AND project_id = ?2",
                params![client.id, project.id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(pc_count, 1);
    }

    /// 测试 create_client：不关联任何项目
    #[test]
    fn test_create_client_no_projects() {
        let dir = TempDir::new().unwrap();
        let db = Database::init(dir.path()).unwrap();

        // 创建客户，不关联任何项目
        let client = db.create_client("独立客户", &[]).unwrap();
        assert_eq!(client.name, "独立客户");

        // 验证 project_clients 中无关联记录
        let pc_count: i64 = db
            .conn()
            .query_row(
                "SELECT COUNT(*) FROM project_clients WHERE client_id = ?1",
                params![client.id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(pc_count, 0);
    }

    /// 测试 create_client：关联多个项目
    #[test]
    fn test_create_client_multiple_projects() {
        let dir = TempDir::new().unwrap();
        let db = Database::init(dir.path()).unwrap();

        let cat = db.create_category("分类", None).unwrap();
        let repo1 = TempDir::new().unwrap();
        let repo2 = TempDir::new().unwrap();
        let p1 = db
            .create_project("项目A", cat.id, repo1.path().to_str().unwrap(), "fastapi")
            .unwrap();
        let p2 = db
            .create_project("项目B", cat.id, repo2.path().to_str().unwrap(), "vue3")
            .unwrap();

        // 创建客户并关联到两个项目
        let client = db.create_client("多项目客户", &[p1.id, p2.id]).unwrap();

        // 验证两条关联记录都已创建
        let pc_count: i64 = db
            .conn()
            .query_row(
                "SELECT COUNT(*) FROM project_clients WHERE client_id = ?1",
                params![client.id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(pc_count, 2);
    }

    /// 测试 list_clients_by_project：按项目过滤客户
    #[test]
    fn test_list_clients_by_project() {
        let dir = TempDir::new().unwrap();
        let db = Database::init(dir.path()).unwrap();

        let cat = db.create_category("分类", None).unwrap();
        let repo1 = TempDir::new().unwrap();
        let repo2 = TempDir::new().unwrap();
        let p1 = db
            .create_project("项目A", cat.id, repo1.path().to_str().unwrap(), "fastapi")
            .unwrap();
        let p2 = db
            .create_project("项目B", cat.id, repo2.path().to_str().unwrap(), "vue3")
            .unwrap();

        // 客户1 关联到项目A
        db.create_client("客户1", &[p1.id]).unwrap();
        // 客户2 关联到项目B
        db.create_client("客户2", &[p2.id]).unwrap();
        // 客户3 关联到两个项目
        db.create_client("客户3", &[p1.id, p2.id]).unwrap();

        // 查询项目A的客户：应返回客户1和客户3
        let clients_a = db.list_clients_by_project(p1.id).unwrap();
        assert_eq!(clients_a.len(), 2);
        let names_a: Vec<&str> = clients_a.iter().map(|c| c.name.as_str()).collect();
        assert!(names_a.contains(&"客户1"));
        assert!(names_a.contains(&"客户3"));

        // 查询项目B的客户：应返回客户2和客户3
        let clients_b = db.list_clients_by_project(p2.id).unwrap();
        assert_eq!(clients_b.len(), 2);
        let names_b: Vec<&str> = clients_b.iter().map(|c| c.name.as_str()).collect();
        assert!(names_b.contains(&"客户2"));
        assert!(names_b.contains(&"客户3"));
    }

    /// 测试 list_clients_by_project：无关联客户时返回空列表
    #[test]
    fn test_list_clients_by_project_empty() {
        let dir = TempDir::new().unwrap();
        let db = Database::init(dir.path()).unwrap();

        let cat = db.create_category("分类", None).unwrap();
        let repo = TempDir::new().unwrap();
        let project = db
            .create_project("项目", cat.id, repo.path().to_str().unwrap(), "fastapi")
            .unwrap();

        // 未创建任何客户，查询应返回空列表
        let clients = db.list_clients_by_project(project.id).unwrap();
        assert!(clients.is_empty());
    }

    /// 测试 update_client：正常更新名称
    #[test]
    fn test_update_client_success() {
        let dir = TempDir::new().unwrap();
        let db = Database::init(dir.path()).unwrap();

        let client = db.create_client("旧名称", &[]).unwrap();
        db.update_client(client.id, "新名称").unwrap();

        // 验证名称已更新
        let name: String = db
            .conn()
            .query_row(
                "SELECT name FROM clients WHERE id = ?1",
                params![client.id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(name, "新名称");
    }

    /// 测试 update_client：不存在的 ID
    #[test]
    fn test_update_client_not_found() {
        let dir = TempDir::new().unwrap();
        let db = Database::init(dir.path()).unwrap();

        let err = db.update_client(999, "名称").unwrap_err();
        assert!(err.contains("不存在"));
    }

    /// 测试 delete_client：正常删除
    #[test]
    fn test_delete_client_success() {
        let dir = TempDir::new().unwrap();
        let db = Database::init(dir.path()).unwrap();

        let client = db.create_client("待删除", &[]).unwrap();
        db.delete_client(client.id).unwrap();

        // 验证客户已被删除
        let count: i64 = db
            .conn()
            .query_row(
                "SELECT COUNT(*) FROM clients WHERE id = ?1",
                params![client.id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 0);
    }

    /// 测试 delete_client：不存在的 ID
    #[test]
    fn test_delete_client_not_found() {
        let dir = TempDir::new().unwrap();
        let db = Database::init(dir.path()).unwrap();

        let err = db.delete_client(999).unwrap_err();
        assert!(err.contains("不存在"));
    }

    /// 测试 delete_client：级联删除 project_clients 关联记录
    #[test]
    fn test_delete_client_cascade_associations() {
        let dir = TempDir::new().unwrap();
        let db = Database::init(dir.path()).unwrap();

        let cat = db.create_category("分类", None).unwrap();
        let repo = TempDir::new().unwrap();
        let project = db
            .create_project("项目", cat.id, repo.path().to_str().unwrap(), "fastapi")
            .unwrap();

        // 创建客户并关联到项目
        let client = db.create_client("客户", &[project.id]).unwrap();

        // 验证关联存在
        let pc_before: i64 = db
            .conn()
            .query_row(
                "SELECT COUNT(*) FROM project_clients WHERE client_id = ?1",
                params![client.id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(pc_before, 1);

        // 删除客户
        db.delete_client(client.id).unwrap();

        // 验证 project_clients 关联记录已被级联删除
        let pc_after: i64 = db
            .conn()
            .query_row(
                "SELECT COUNT(*) FROM project_clients WHERE client_id = ?1",
                params![client.id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(pc_after, 0);
    }

    // ========================================================================
    // Category CRUD 属性测试 (Property-Based Tests)
    // ========================================================================

    /// 生成可选的分类描述策略
    fn optional_description_strategy() -> impl Strategy<Value = Option<String>> {
        prop_oneof![Just(None), "[a-zA-Z0-9_ ]{1,50}".prop_map(Some),]
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// Feature: prism-console-v2, Property 1: Category CRUD Round-Trip
        ///
        /// 对于任意合法的分类名称和可选描述，创建分类后列出所有分类应包含
        /// 同名同描述的分类。更新该分类名称后读取应反映更新。删除后列出应不再包含。
        ///
        /// **Validates: Requirements 1.1, 1.2**
        #[test]
        fn prop_category_crud_round_trip(
            name in "[a-zA-Z][a-zA-Z0-9_]{1,30}",
            description in optional_description_strategy(),
            updated_name in "[a-zA-Z][a-zA-Z0-9_]{1,30}"
        ) {
            let dir = TempDir::new().unwrap();
            let db = Database::init(dir.path()).unwrap();

            // 1. 创建分类
            let cat = db.create_category(&name, description.as_deref()).unwrap();
            prop_assert_eq!(&cat.name, &name);
            prop_assert_eq!(&cat.description, &description);

            // 2. 列出所有分类，应包含刚创建的分类
            let cats = db.list_categories().unwrap();
            let found = cats.iter().find(|c| c.id == cat.id);
            prop_assert!(found.is_some(), "创建后列表中应包含该分类");
            let found = found.unwrap();
            prop_assert_eq!(&found.name, &name);
            prop_assert_eq!(&found.description, &description);

            // 3. 更新分类名称
            db.update_category(cat.id, &updated_name, description.as_deref()).unwrap();

            // 4. 再次列出，验证名称已更新
            let cats_after_update = db.list_categories().unwrap();
            let updated = cats_after_update.iter().find(|c| c.id == cat.id);
            prop_assert!(updated.is_some(), "更新后列表中应仍包含该分类");
            prop_assert_eq!(&updated.unwrap().name, &updated_name);

            // 5. 删除分类
            db.delete_category(cat.id).unwrap();

            // 6. 列出所有分类，应不再包含已删除的分类
            let cats_after_delete = db.list_categories().unwrap();
            let deleted = cats_after_delete.iter().find(|c| c.id == cat.id);
            prop_assert!(deleted.is_none(), "删除后列表中不应包含该分类");
        }

        /// Feature: prism-console-v2, Property 2: Duplicate Category Name Rejection
        ///
        /// 对于任意已存在于数据库中的分类名称，尝试再次创建同名分类应返回错误，
        /// 且分类总数保持不变。
        ///
        /// **Validates: Requirements 1.3**
        #[test]
        fn prop_duplicate_category_name_rejection(
            name in "[a-zA-Z][a-zA-Z0-9_]{1,30}",
            desc1 in optional_description_strategy(),
            desc2 in optional_description_strategy()
        ) {
            let dir = TempDir::new().unwrap();
            let db = Database::init(dir.path()).unwrap();

            // 1. 创建第一个分类
            db.create_category(&name, desc1.as_deref()).unwrap();

            // 2. 记录当前分类总数
            let count_before = db.list_categories().unwrap().len();

            // 3. 尝试创建同名分类，应返回错误
            let result = db.create_category(&name, desc2.as_deref());
            prop_assert!(result.is_err(), "创建重复名称的分类应返回错误");
            prop_assert_eq!(result.unwrap_err(), "分类名称已存在".to_string());

            // 4. 分类总数应保持不变
            let count_after = db.list_categories().unwrap().len();
            prop_assert_eq!(count_before, count_after, "重复创建后分类总数应不变");
        }

        /// Feature: prism-console-v2, Property 3: Category With Projects Cannot Be Deleted
        ///
        /// 对于任意至少关联了一个项目的分类，尝试删除该分类应失败并返回错误，
        /// 且该分类在数据库中仍然存在。
        ///
        /// **Validates: Requirements 1.4**
        #[test]
        fn prop_category_with_projects_cannot_be_deleted(
            cat_name in "[a-zA-Z][a-zA-Z0-9_]{1,30}",
            project_name in "[a-zA-Z][a-zA-Z0-9_]{1,30}"
        ) {
            let dir = TempDir::new().unwrap();
            let db = Database::init(dir.path()).unwrap();

            // 1. 创建分类
            let cat = db.create_category(&cat_name, None).unwrap();

            // 2. 手动插入一个关联到该分类的项目记录
            db.conn()
                .execute(
                    "INSERT INTO projects (name, category_id, repo_path, tech_stack_type) VALUES (?1, ?2, ?3, ?4)",
                    params![project_name, cat.id, "/tmp/fake/path", "fastapi"],
                )
                .unwrap();

            // 3. 尝试删除该分类，应失败
            let result = db.delete_category(cat.id);
            prop_assert!(result.is_err(), "删除含项目的分类应返回错误");
            prop_assert_eq!(result.unwrap_err(), "该分类下仍有项目，无法删除".to_string());

            // 4. 验证分类仍然存在于数据库中
            let cats = db.list_categories().unwrap();
            let still_exists = cats.iter().find(|c| c.id == cat.id);
            prop_assert!(still_exists.is_some(), "删除失败后分类应仍然存在");
            prop_assert_eq!(&still_exists.unwrap().name, &cat_name);
        }

        // ====================================================================
        // Project CRUD 属性测试 (Property-Based Tests)
        // ====================================================================

        /// Feature: prism-console-v2, Property 4: Project CRUD Round-Trip
        ///
        /// 对于任意合法的项目数据（名称、分类ID、仓库路径、技术栈类型），
        /// 创建项目后读取应返回相同的字段值。更新项目的名称、分类或技术栈类型
        /// 后读取应反映更新后的值。
        ///
        /// **Validates: Requirements 2.1, 2.4, 2.7**
        #[test]
        fn prop_project_crud_round_trip(
            name in "[a-zA-Z][a-zA-Z0-9_]{1,30}",
            cat_name in "[a-zA-Z][a-zA-Z0-9_]{1,30}",
            tech_stack in prop_oneof![Just("fastapi".to_string()), Just("vue3".to_string())],
            updated_name in "[a-zA-Z][a-zA-Z0-9_]{1,30}",
            updated_cat_name in "[a-zA-Z][a-zA-Z0-9_]{1,30}",
            updated_tech in prop_oneof![Just("fastapi".to_string()), Just("vue3".to_string())],
        ) {
            // 使用 TempDir 作为合法的仓库路径（确保路径存在于文件系统）
            let db_dir = TempDir::new().unwrap();
            let repo_dir = TempDir::new().unwrap();
            let db = Database::init(db_dir.path()).unwrap();
            let repo_path = repo_dir.path().to_str().unwrap();

            // 1. 创建分类（项目需要关联分类）
            let cat = db.create_category(&cat_name, None).unwrap();

            // 2. 创建项目
            let project = db.create_project(&name, cat.id, repo_path, &tech_stack).unwrap();

            // 3. 验证创建后的字段值与输入一致
            prop_assert_eq!(&project.name, &name);
            prop_assert_eq!(project.category_id, cat.id);
            prop_assert_eq!(&project.repo_path, repo_path);
            prop_assert_eq!(&project.tech_stack_type, &tech_stack);

            // 4. 通过 get_project 读取，验证 round-trip 一致性
            let fetched = db.get_project(project.id).unwrap();
            prop_assert_eq!(&fetched.name, &name);
            prop_assert_eq!(fetched.category_id, cat.id);
            prop_assert_eq!(&fetched.repo_path, repo_path);
            prop_assert_eq!(&fetched.tech_stack_type, &tech_stack);

            // 5. 创建第二个分类用于更新测试
            // 使用不同的名称避免与第一个分类重名
            let updated_cat_unique = format!("upd_{}", &updated_cat_name);
            let cat2 = db.create_category(&updated_cat_unique, None).unwrap();

            // 6. 更新项目的名称、分类和技术栈类型
            db.update_project(project.id, &updated_name, cat2.id, &updated_tech).unwrap();

            // 7. 再次读取，验证更新后的值
            let updated_project = db.get_project(project.id).unwrap();
            prop_assert_eq!(&updated_project.name, &updated_name);
            prop_assert_eq!(updated_project.category_id, cat2.id);
            prop_assert_eq!(&updated_project.tech_stack_type, &updated_tech);
            // repo_path 不应被更新（update_project 不修改 repo_path）
            prop_assert_eq!(&updated_project.repo_path, repo_path);
        }

        /// Feature: prism-console-v2, Property 5: Project Path Validation
        ///
        /// 对于任意文件系统路径字符串，使用该路径创建项目时，仅当路径存在于
        /// 文件系统时才应成功。如果路径不存在，创建应失败并返回描述性错误，
        /// 且不应有项目记录被持久化。
        ///
        /// **Validates: Requirements 2.2, 2.3**
        #[test]
        fn prop_project_path_validation(
            name in "[a-zA-Z][a-zA-Z0-9_]{1,30}",
            cat_name in "[a-zA-Z][a-zA-Z0-9_]{1,30}",
            // 生成随机路径片段，拼接为不存在的路径
            fake_segment in "[a-zA-Z][a-zA-Z0-9]{1,20}",
        ) {
            let db_dir = TempDir::new().unwrap();
            let db = Database::init(db_dir.path()).unwrap();

            // 创建分类
            let cat = db.create_category(&cat_name, None).unwrap();

            // --- 测试不存在的路径 ---
            // 构造一个确保不存在的路径
            let non_existent_path = format!("/tmp/prism_test_nonexistent_{}", fake_segment);
            // 确保路径确实不存在
            if !std::path::Path::new(&non_existent_path).exists() {
                let result = db.create_project(&name, cat.id, &non_existent_path, "fastapi");
                prop_assert!(result.is_err(), "不存在的路径应导致创建失败");
                let err_msg = result.unwrap_err();
                prop_assert!(
                    err_msg.contains("项目路径不存在"),
                    "错误信息应包含 '项目路径不存在'，实际为：{}", err_msg
                );

                // 验证没有项目记录被持久化
                let projects = db.list_projects().unwrap();
                prop_assert!(
                    projects.is_empty(),
                    "路径不存在时不应有项目记录被持久化"
                );
            }

            // --- 测试存在的路径 ---
            let valid_dir = TempDir::new().unwrap();
            let valid_path = valid_dir.path().to_str().unwrap();
            let result = db.create_project(&name, cat.id, valid_path, "fastapi");
            prop_assert!(result.is_ok(), "存在的路径应允许创建项目成功");

            // 验证项目确实被持久化
            let projects = db.list_projects().unwrap();
            prop_assert_eq!(projects.len(), 1, "成功创建后应有一条项目记录");
            prop_assert_eq!(&projects[0].repo_path, valid_path);
        }

        /// Feature: prism-console-v2, Property 6: Project Cascade Delete
        ///
        /// 对于任意拥有关联客户绑定和构建记录的项目，删除该项目后，
        /// 项目本身、其客户绑定（project_clients 表）和构建记录
        /// 都应从数据库中消失。
        ///
        /// **Validates: Requirements 2.5**
        #[test]
        fn prop_project_cascade_delete(
            project_name in "[a-zA-Z][a-zA-Z0-9_]{1,30}",
            cat_name in "[a-zA-Z][a-zA-Z0-9_]{1,30}",
            client_name in "[a-zA-Z][a-zA-Z0-9_]{1,30}",
            tech_stack in prop_oneof![Just("fastapi".to_string()), Just("vue3".to_string())],
            modules_json in Just("[\"mod_a\",\"mod_b\"]".to_string()),
        ) {
            let db_dir = TempDir::new().unwrap();
            let repo_dir = TempDir::new().unwrap();
            let db = Database::init(db_dir.path()).unwrap();
            let repo_path = repo_dir.path().to_str().unwrap();

            // 1. 创建分类和项目
            let cat = db.create_category(&cat_name, None).unwrap();
            let project = db.create_project(&project_name, cat.id, repo_path, &tech_stack).unwrap();

            // 2. 手动插入客户记录（create_client 方法尚未实现）
            db.conn()
                .execute(
                    "INSERT INTO clients (name) VALUES (?1)",
                    params![client_name],
                )
                .unwrap();
            let client_id: i64 = db.conn().last_insert_rowid();

            // 3. 手动插入项目-客户关联记录
            db.conn()
                .execute(
                    "INSERT INTO project_clients (project_id, client_id) VALUES (?1, ?2)",
                    params![project.id, client_id],
                )
                .unwrap();

            // 4. 手动插入构建记录
            db.conn()
                .execute(
                    "INSERT INTO build_records (project_id, client_id, selected_modules, output_path) VALUES (?1, ?2, ?3, ?4)",
                    params![project.id, client_id, modules_json, "/tmp/output.zip"],
                )
                .unwrap();

            // 5. 验证关联数据确实存在
            let pc_count: i64 = db.conn()
                .query_row(
                    "SELECT COUNT(*) FROM project_clients WHERE project_id = ?1",
                    params![project.id],
                    |row| row.get(0),
                )
                .unwrap();
            prop_assert_eq!(pc_count, 1, "删除前应有 1 条项目-客户关联记录");

            let br_count: i64 = db.conn()
                .query_row(
                    "SELECT COUNT(*) FROM build_records WHERE project_id = ?1",
                    params![project.id],
                    |row| row.get(0),
                )
                .unwrap();
            prop_assert_eq!(br_count, 1, "删除前应有 1 条构建记录");

            // 6. 删除项目
            db.delete_project(project.id).unwrap();

            // 7. 验证项目已被删除
            let project_result = db.get_project(project.id);
            prop_assert!(project_result.is_err(), "删除后项目应不存在");

            // 8. 验证项目-客户关联记录已被级联删除
            let pc_count_after: i64 = db.conn()
                .query_row(
                    "SELECT COUNT(*) FROM project_clients WHERE project_id = ?1",
                    params![project.id],
                    |row| row.get(0),
                )
                .unwrap();
            prop_assert_eq!(pc_count_after, 0, "删除后项目-客户关联记录应为 0");

            // 9. 验证构建记录已被级联删除
            let br_count_after: i64 = db.conn()
                .query_row(
                    "SELECT COUNT(*) FROM build_records WHERE project_id = ?1",
                    params![project.id],
                    |row| row.get(0),
                )
                .unwrap();
            prop_assert_eq!(br_count_after, 0, "删除后构建记录应为 0");

            // 10. 验证客户本身不应被删除（仅关联关系被删除）
            let client_count: i64 = db.conn()
                .query_row(
                    "SELECT COUNT(*) FROM clients WHERE id = ?1",
                    params![client_id],
                    |row| row.get(0),
                )
                .unwrap();
            prop_assert_eq!(client_count, 1, "客户本身不应被级联删除");
        }

        // ====================================================================
        // Client CRUD 属性测试 (Property-Based Tests)
        // ====================================================================

        /// Feature: prism-console-v2, Property 7: Client CRUD Round-Trip
        ///
        /// 对于任意合法的客户名称，创建客户后通过项目查询应包含该客户。
        /// 更新客户名称后读取应反映更新。删除后应从列表中移除。
        ///
        /// **Validates: Requirements 3.1**
        #[test]
        fn prop_client_crud_round_trip(
            client_name in "[a-zA-Z][a-zA-Z0-9_]{1,30}",
            updated_name in "[a-zA-Z][a-zA-Z0-9_]{1,30}",
            cat_name in "[a-zA-Z][a-zA-Z0-9_]{1,30}",
            project_name in "[a-zA-Z][a-zA-Z0-9_]{1,30}",
        ) {
            let db_dir = TempDir::new().unwrap();
            let repo_dir = TempDir::new().unwrap();
            let db = Database::init(db_dir.path()).unwrap();
            let repo_path = repo_dir.path().to_str().unwrap();

            // 1. 创建分类和项目（客户需要关联到项目才能通过 list_clients_by_project 查询）
            let cat = db.create_category(&cat_name, None).unwrap();
            let project = db.create_project(&project_name, cat.id, repo_path, "fastapi").unwrap();

            // 2. 创建客户并关联到项目
            let client = db.create_client(&client_name, &[project.id]).unwrap();
            prop_assert_eq!(&client.name, &client_name);

            // 3. 通过项目查询客户列表，应包含刚创建的客户
            let clients = db.list_clients_by_project(project.id).unwrap();
            let found = clients.iter().find(|c| c.id == client.id);
            prop_assert!(found.is_some(), "创建后列表中应包含该客户");
            prop_assert_eq!(&found.unwrap().name, &client_name);

            // 4. 更新客户名称
            db.update_client(client.id, &updated_name).unwrap();

            // 5. 再次查询，验证名称已更新
            let clients_after_update = db.list_clients_by_project(project.id).unwrap();
            let updated = clients_after_update.iter().find(|c| c.id == client.id);
            prop_assert!(updated.is_some(), "更新后列表中应仍包含该客户");
            prop_assert_eq!(&updated.unwrap().name, &updated_name);

            // 6. 删除客户
            db.delete_client(client.id).unwrap();

            // 7. 再次查询，应不再包含已删除的客户
            let clients_after_delete = db.list_clients_by_project(project.id).unwrap();
            let deleted = clients_after_delete.iter().find(|c| c.id == client.id);
            prop_assert!(deleted.is_none(), "删除后列表中不应包含该客户");
        }

        /// Feature: prism-console-v2, Property 8: Client-Project Association Round-Trip
        ///
        /// 对于任意客户和任意非空的项目 ID 集合，创建客户并关联这些项目后，
        /// 通过任意一个关联的项目 ID 查询客户列表都应包含该客户。
        ///
        /// **Validates: Requirements 3.2**
        #[test]
        fn prop_client_project_association_round_trip(
            client_name in "[a-zA-Z][a-zA-Z0-9_]{1,30}",
            cat_name in "[a-zA-Z][a-zA-Z0-9_]{1,30}",
            // 生成 1~4 个项目名称（非空集合）
            project_names in prop::collection::vec("[a-zA-Z][a-zA-Z0-9_]{1,20}", 1..=4),
        ) {
            let db_dir = TempDir::new().unwrap();
            let db = Database::init(db_dir.path()).unwrap();

            // 1. 创建分类
            let cat = db.create_category(&cat_name, None).unwrap();

            // 2. 为每个项目名称创建项目（每个项目需要独立的 TempDir 作为 repo_path）
            let mut project_ids = Vec::new();
            let mut _repo_dirs = Vec::new(); // 保持 TempDir 存活，防止路径被清理
            for (i, pname) in project_names.iter().enumerate() {
                let repo_dir = TempDir::new().unwrap();
                let repo_path = repo_dir.path().to_str().unwrap().to_string();
                // 使用索引后缀确保项目名称唯一
                let unique_name = format!("{}_{}", pname, i);
                let project = db.create_project(&unique_name, cat.id, &repo_path, "fastapi").unwrap();
                project_ids.push(project.id);
                _repo_dirs.push(repo_dir);
            }

            // 3. 创建客户并关联到所有项目
            let client = db.create_client(&client_name, &project_ids).unwrap();

            // 4. 通过每个项目 ID 查询，都应包含该客户
            for &pid in &project_ids {
                let clients = db.list_clients_by_project(pid).unwrap();
                let found = clients.iter().find(|c| c.id == client.id);
                prop_assert!(
                    found.is_some(),
                    "通过项目 ID {} 查询应包含客户 '{}'", pid, client_name
                );
                prop_assert_eq!(&found.unwrap().name, &client_name);
            }
        }

        /// Feature: prism-console-v2, Property 9: Client Filtering By Project
        ///
        /// 对于任意项目，通过该项目查询客户列表应仅返回与该项目有关联的客户。
        /// 没有与该项目关联的客户不应出现在结果中。
        ///
        /// **Validates: Requirements 3.3**
        #[test]
        fn prop_client_filtering_by_project(
            cat_name in "[a-zA-Z][a-zA-Z0-9_]{1,30}",
            client_a_name in "[a-zA-Z][a-zA-Z0-9_]{1,30}",
            client_b_name in "[a-zA-Z][a-zA-Z0-9_]{1,30}",
            project_a_name in "[a-zA-Z][a-zA-Z0-9_]{1,30}",
            project_b_name in "[a-zA-Z][a-zA-Z0-9_]{1,30}",
        ) {
            let db_dir = TempDir::new().unwrap();
            let repo_dir_a = TempDir::new().unwrap();
            let repo_dir_b = TempDir::new().unwrap();
            let db = Database::init(db_dir.path()).unwrap();
            let repo_path_a = repo_dir_a.path().to_str().unwrap();
            let repo_path_b = repo_dir_b.path().to_str().unwrap();

            // 1. 创建分类
            let cat = db.create_category(&cat_name, None).unwrap();

            // 2. 创建两个项目（使用前缀确保名称唯一）
            let project_a = db.create_project(
                &format!("pa_{}", project_a_name), cat.id, repo_path_a, "fastapi"
            ).unwrap();
            let project_b = db.create_project(
                &format!("pb_{}", project_b_name), cat.id, repo_path_b, "vue3"
            ).unwrap();

            // 3. 创建客户 A 仅关联到项目 A
            let client_a = db.create_client(
                &format!("ca_{}", client_a_name), &[project_a.id]
            ).unwrap();

            // 4. 创建客户 B 仅关联到项目 B
            let client_b = db.create_client(
                &format!("cb_{}", client_b_name), &[project_b.id]
            ).unwrap();

            // 5. 查询项目 A 的客户列表
            let clients_for_a = db.list_clients_by_project(project_a.id).unwrap();

            // 6. 验证：项目 A 的客户列表应包含客户 A
            let has_client_a = clients_for_a.iter().any(|c| c.id == client_a.id);
            prop_assert!(has_client_a, "项目 A 的客户列表应包含客户 A");

            // 7. 验证：项目 A 的客户列表不应包含客户 B
            let has_client_b = clients_for_a.iter().any(|c| c.id == client_b.id);
            prop_assert!(!has_client_b, "项目 A 的客户列表不应包含客户 B");

            // 8. 查询项目 B 的客户列表
            let clients_for_b = db.list_clients_by_project(project_b.id).unwrap();

            // 9. 验证：项目 B 的客户列表应包含客户 B
            let has_client_b_in_b = clients_for_b.iter().any(|c| c.id == client_b.id);
            prop_assert!(has_client_b_in_b, "项目 B 的客户列表应包含客户 B");

            // 10. 验证：项目 B 的客户列表不应包含客户 A
            let has_client_a_in_b = clients_for_b.iter().any(|c| c.id == client_a.id);
            prop_assert!(!has_client_a_in_b, "项目 B 的客户列表不应包含客户 A");

            // 11. 额外验证：所有返回的客户都确实在 project_clients 表中有关联
            for client in &clients_for_a {
                let assoc_count: i64 = db.conn()
                    .query_row(
                        "SELECT COUNT(*) FROM project_clients WHERE project_id = ?1 AND client_id = ?2",
                        params![project_a.id, client.id],
                        |row| row.get(0),
                    )
                    .unwrap();
                prop_assert_eq!(
                    assoc_count, 1,
                    "返回的客户 {} 应在 project_clients 表中有关联记录", client.id
                );
            }
        }

        // ====================================================================
        // Build Record 和 Settings 属性测试 (Property-Based Tests)
        // ====================================================================

        /// Feature: prism-console-v2, Property 13: Build Creates Database Record
        ///
        /// 对于任意成功的构建操作，数据库中应存在对应的构建记录，
        /// 且 project_id、client_id、selected_modules JSON 和 output_path 均正确。
        /// selected_modules JSON 应能正确往返：解析存储的 JSON 应产生原始的模块名称列表。
        ///
        /// **Validates: Requirements 6.3**
        #[test]
        fn prop_build_creates_database_record(
            cat_name in "[a-zA-Z][a-zA-Z0-9_]{1,30}",
            project_name in "[a-zA-Z][a-zA-Z0-9_]{1,30}",
            client_name in "[a-zA-Z][a-zA-Z0-9_]{1,30}",
            // 生成 1~5 个随机模块名称
            module_names in prop::collection::vec("[a-zA-Z][a-zA-Z0-9_]{1,20}", 1..=5),
            output_suffix in "[a-zA-Z0-9_]{1,20}",
        ) {
            let db_dir = TempDir::new().unwrap();
            let repo_dir = TempDir::new().unwrap();
            let db = Database::init(db_dir.path()).unwrap();
            let repo_path = repo_dir.path().to_str().unwrap();

            // 1. 创建分类、项目和客户（构建记录的前置依赖）
            let cat = db.create_category(&cat_name, None).unwrap();
            let project = db.create_project(&project_name, cat.id, repo_path, "fastapi").unwrap();
            let client = db.create_client(&client_name, &[project.id]).unwrap();

            // 2. 将模块名称列表序列化为 JSON 字符串
            let modules_json = serde_json::to_string(&module_names).unwrap();
            let output_path = format!("/tmp/build_{}.zip", output_suffix);

            // 3. 创建构建记录
            let record = db.create_build_record(
                project.id, client.id, &modules_json, &output_path
            ).unwrap();

            // 4. 验证返回的构建记录字段与输入一致
            prop_assert_eq!(record.project_id, project.id, "project_id 应匹配");
            prop_assert_eq!(record.client_id, client.id, "client_id 应匹配");
            prop_assert_eq!(&record.selected_modules, &modules_json, "selected_modules JSON 应匹配");
            prop_assert_eq!(&record.output_path, &output_path, "output_path 应匹配");

            // 5. 通过 list_build_records_by_project 查询，验证记录存在于数据库中
            let records = db.list_build_records_by_project(project.id).unwrap();
            let found = records.iter().find(|r| r.id == record.id);
            prop_assert!(found.is_some(), "构建记录应存在于数据库中");
            let found = found.unwrap();
            prop_assert_eq!(found.project_id, project.id);
            prop_assert_eq!(found.client_id, client.id);
            prop_assert_eq!(&found.output_path, &output_path);

            // 6. JSON 往返验证：解析存储的 JSON 应产生原始的模块名称列表
            let parsed_modules: Vec<String> = serde_json::from_str(&found.selected_modules).unwrap();
            prop_assert_eq!(
                &parsed_modules, &module_names,
                "解析存储的 JSON 应产生原始的模块名称列表"
            );
        }

        /// Feature: prism-console-v2, Property 16: Build History Filtering By Project
        ///
        /// 对于任意拥有构建记录的项目，按该项目查询构建记录应仅返回
        /// project_id 匹配的记录。属于其他项目的记录不应出现。
        ///
        /// **Validates: Requirements 9.5**
        #[test]
        fn prop_build_history_filtering_by_project(
            cat_name in "[a-zA-Z][a-zA-Z0-9_]{1,30}",
            project_a_name in "[a-zA-Z][a-zA-Z0-9_]{1,30}",
            project_b_name in "[a-zA-Z][a-zA-Z0-9_]{1,30}",
            client_name in "[a-zA-Z][a-zA-Z0-9_]{1,30}",
            // 为每个项目生成 1~3 条构建记录的模块数据
            modules_a_count in 1usize..=3,
            modules_b_count in 1usize..=3,
        ) {
            let db_dir = TempDir::new().unwrap();
            let repo_dir_a = TempDir::new().unwrap();
            let repo_dir_b = TempDir::new().unwrap();
            let db = Database::init(db_dir.path()).unwrap();
            let repo_path_a = repo_dir_a.path().to_str().unwrap();
            let repo_path_b = repo_dir_b.path().to_str().unwrap();

            // 1. 创建分类和两个项目
            let cat = db.create_category(&cat_name, None).unwrap();
            let project_a = db.create_project(
                &format!("pa_{}", project_a_name), cat.id, repo_path_a, "fastapi"
            ).unwrap();
            let project_b = db.create_project(
                &format!("pb_{}", project_b_name), cat.id, repo_path_b, "vue3"
            ).unwrap();

            // 2. 创建客户并关联到两个项目
            let client = db.create_client(&client_name, &[project_a.id, project_b.id]).unwrap();

            // 3. 为项目 A 创建多条构建记录
            let mut records_a_ids = Vec::new();
            for i in 0..modules_a_count {
                let modules_json = format!("[\"mod_a_{}\"]", i);
                let output_path = format!("/tmp/build_a_{}.zip", i);
                let record = db.create_build_record(
                    project_a.id, client.id, &modules_json, &output_path
                ).unwrap();
                records_a_ids.push(record.id);
            }

            // 4. 为项目 B 创建多条构建记录
            let mut records_b_ids = Vec::new();
            for i in 0..modules_b_count {
                let modules_json = format!("[\"mod_b_{}\"]", i);
                let output_path = format!("/tmp/build_b_{}.zip", i);
                let record = db.create_build_record(
                    project_b.id, client.id, &modules_json, &output_path
                ).unwrap();
                records_b_ids.push(record.id);
            }

            // 5. 查询项目 A 的构建记录
            let results_a = db.list_build_records_by_project(project_a.id).unwrap();

            // 6. 验证：返回的记录数量应等于为项目 A 创建的记录数
            prop_assert_eq!(
                results_a.len(), modules_a_count,
                "项目 A 的构建记录数量应为 {}", modules_a_count
            );

            // 7. 验证：所有返回的记录的 project_id 都应匹配项目 A
            for record in &results_a {
                prop_assert_eq!(
                    record.project_id, project_a.id,
                    "项目 A 的构建记录 project_id 应匹配"
                );
            }

            // 8. 验证：项目 A 的查询结果不应包含项目 B 的记录
            for record in &results_a {
                prop_assert!(
                    !records_b_ids.contains(&record.id),
                    "项目 A 的查询结果不应包含项目 B 的记录 ID {}", record.id
                );
            }

            // 9. 查询项目 B 的构建记录，做同样的验证
            let results_b = db.list_build_records_by_project(project_b.id).unwrap();
            prop_assert_eq!(
                results_b.len(), modules_b_count,
                "项目 B 的构建记录数量应为 {}", modules_b_count
            );
            for record in &results_b {
                prop_assert_eq!(
                    record.project_id, project_b.id,
                    "项目 B 的构建记录 project_id 应匹配"
                );
                prop_assert!(
                    !records_a_ids.contains(&record.id),
                    "项目 B 的查询结果不应包含项目 A 的记录 ID {}", record.id
                );
            }
        }

        /// Feature: prism-console-v2, Property 17: Settings Round-Trip
        ///
        /// 对于任意设置键值对，保存设置后加载所有设置应包含已保存的键及相同的值。
        /// 更新值后再次加载应反映更新。
        ///
        /// **Validates: Requirements 10.3**
        #[test]
        fn prop_settings_round_trip(
            value1 in "[a-zA-Z0-9_/\\-]{1,100}",
            value2 in "[a-zA-Z0-9_/\\-]{1,100}",
        ) {
            let db_dir = TempDir::new().unwrap();
            let db = Database::init(db_dir.path()).unwrap();
            let db_path = db_dir.path().join("prism.db").to_str().unwrap().to_string();

            // 使用 default_output_dir 键进行测试（get_settings 支持的键）
            let key = "default_output_dir";

            // 1. 保存设置
            db.save_setting(key, &value1).unwrap();

            // 2. 加载设置，验证值与保存的一致
            let settings = db.get_settings(&db_path).unwrap();
            prop_assert_eq!(
                settings.default_output_dir.as_deref(),
                Some(value1.as_str()),
                "保存后加载的设置值应与保存的一致"
            );

            // 3. 更新设置值
            db.save_setting(key, &value2).unwrap();

            // 4. 再次加载，验证值已更新
            let settings_updated = db.get_settings(&db_path).unwrap();
            prop_assert_eq!(
                settings_updated.default_output_dir.as_deref(),
                Some(value2.as_str()),
                "更新后加载的设置值应反映更新"
            );

            // 5. 额外验证：通过直接 SQL 查询确认数据库中的值
            let stored_value: String = db.conn()
                .query_row(
                    "SELECT value FROM settings WHERE key = ?1",
                    params![key],
                    |row| row.get(0),
                )
                .unwrap();
            prop_assert_eq!(
                &stored_value, &value2,
                "数据库中存储的值应与最后一次保存的值一致"
            );
        }

        // ====================================================================
        // 数据库持久化属性测试 (Property-Based Tests)
        // ====================================================================

        // Feature: prism-console-v2, Property 10: Database Persistence Across Restarts
        /// Feature: prism-console-v2, Property 10: Database Persistence Across Restarts
        ///
        /// 对于任意写入数据库的分类、项目和客户集合，关闭数据库连接后
        /// 从相同的文件路径重新打开数据库，应保留所有先前存储的数据，
        /// 且字段值完全一致。
        ///
        /// **Validates: Requirements 4.7**
        #[test]
        fn prop_database_persistence_across_restarts(
            // 生成 1~3 个分类名称
            cat_names in prop::collection::vec("[a-zA-Z][a-zA-Z0-9_]{1,20}", 1..=3),
            cat_descriptions in prop::collection::vec(optional_description_strategy(), 1..=3),
            // 生成 1~3 个项目名称
            project_names in prop::collection::vec("[a-zA-Z][a-zA-Z0-9_]{1,20}", 1..=3),
            tech_stacks in prop::collection::vec(
                prop_oneof![Just("fastapi".to_string()), Just("vue3".to_string())],
                1..=3
            ),
            // 生成 1~3 个客户名称
            client_names in prop::collection::vec("[a-zA-Z][a-zA-Z0-9_]{1,20}", 1..=3),
        ) {
            // 使用 TempDir 作为数据库目录（不会在测试结束前被清理）
            let db_dir = TempDir::new().unwrap();

            // 用于存储创建的数据，以便在重新打开后验证
            let mut created_categories: Vec<Category> = Vec::new();
            let mut created_projects: Vec<Project> = Vec::new();
            let mut created_clients: Vec<Client> = Vec::new();

            // --- 阶段 1：创建数据库并写入数据 ---
            {
                let db = Database::init(db_dir.path()).unwrap();

                // 1. 创建分类（使用索引后缀确保名称唯一）
                let desc_len = cat_descriptions.len();
                for (i, cat_name) in cat_names.iter().enumerate() {
                    let unique_name = format!("cat_{}_{}", cat_name, i);
                    let desc = if i < desc_len {
                        cat_descriptions[i].as_deref()
                    } else {
                        None
                    };
                    let cat = db.create_category(&unique_name, desc).unwrap();
                    created_categories.push(cat);
                }

                // 2. 创建项目（每个项目需要独立的 TempDir 作为 repo_path）
                let mut _repo_dirs = Vec::new(); // 保持 TempDir 存活
                let proj_len = project_names.len().min(created_categories.len());
                let tech_len = tech_stacks.len();
                for i in 0..proj_len {
                    let repo_dir = TempDir::new().unwrap();
                    let repo_path = repo_dir.path().to_str().unwrap().to_string();
                    let unique_name = format!("proj_{}_{}", project_names[i], i);
                    let tech = if i < tech_len { &tech_stacks[i] } else { "fastapi" };
                    let cat_id = created_categories[i].id;
                    let project = db.create_project(
                        &unique_name, cat_id, &repo_path, tech
                    ).unwrap();
                    created_projects.push(project);
                    _repo_dirs.push(repo_dir);
                }

                // 3. 创建客户并关联到已创建的项目
                if !created_projects.is_empty() {
                    let all_project_ids: Vec<i64> = created_projects.iter().map(|p| p.id).collect();
                    for (i, client_name) in client_names.iter().enumerate() {
                        let unique_name = format!("client_{}_{}", client_name, i);
                        let client = db.create_client(&unique_name, &all_project_ids).unwrap();
                        created_clients.push(client);
                    }
                }

                // db 在此作用域结束时被 drop，关闭数据库连接
            }

            // --- 阶段 2：重新打开数据库并验证数据持久化 ---
            {
                let db2 = Database::init(db_dir.path()).unwrap();

                // 4. 验证分类数据持久化
                let categories = db2.list_categories().unwrap();
                for expected_cat in &created_categories {
                    let found = categories.iter().find(|c| c.id == expected_cat.id);
                    prop_assert!(
                        found.is_some(),
                        "重新打开后应找到分类 '{}' (id={})", expected_cat.name, expected_cat.id
                    );
                    let found = found.unwrap();
                    prop_assert_eq!(
                        &found.name, &expected_cat.name,
                        "分类名称应一致"
                    );
                    prop_assert_eq!(
                        &found.description, &expected_cat.description,
                        "分类描述应一致"
                    );
                    prop_assert_eq!(
                        &found.created_at, &expected_cat.created_at,
                        "分类创建时间应一致"
                    );
                }

                // 5. 验证项目数据持久化
                let projects = db2.list_projects().unwrap();
                for expected_proj in &created_projects {
                    let found = projects.iter().find(|p| p.id == expected_proj.id);
                    prop_assert!(
                        found.is_some(),
                        "重新打开后应找到项目 '{}' (id={})", expected_proj.name, expected_proj.id
                    );
                    let found = found.unwrap();
                    prop_assert_eq!(
                        &found.name, &expected_proj.name,
                        "项目名称应一致"
                    );
                    prop_assert_eq!(
                        found.category_id, expected_proj.category_id,
                        "项目分类 ID 应一致"
                    );
                    prop_assert_eq!(
                        &found.repo_path, &expected_proj.repo_path,
                        "项目仓库路径应一致"
                    );
                    prop_assert_eq!(
                        &found.tech_stack_type, &expected_proj.tech_stack_type,
                        "项目技术栈类型应一致"
                    );
                    prop_assert_eq!(
                        &found.created_at, &expected_proj.created_at,
                        "项目创建时间应一致"
                    );
                    prop_assert_eq!(
                        &found.updated_at, &expected_proj.updated_at,
                        "项目更新时间应一致"
                    );
                }

                // 6. 验证客户数据持久化
                if !created_projects.is_empty() {
                    // 通过第一个项目查询客户列表
                    let first_project_id = created_projects[0].id;
                    let clients = db2.list_clients_by_project(first_project_id).unwrap();
                    for expected_client in &created_clients {
                        let found = clients.iter().find(|c| c.id == expected_client.id);
                        prop_assert!(
                            found.is_some(),
                            "重新打开后应找到客户 '{}' (id={})", expected_client.name, expected_client.id
                        );
                        let found = found.unwrap();
                        prop_assert_eq!(
                            &found.name, &expected_client.name,
                            "客户名称应一致"
                        );
                        prop_assert_eq!(
                            &found.created_at, &expected_client.created_at,
                            "客户创建时间应一致"
                        );
                    }

                    // 7. 验证项目-客户关联关系持久化
                    let all_project_ids: Vec<i64> = created_projects.iter().map(|p| p.id).collect();
                    for &pid in &all_project_ids {
                        let clients_for_project = db2.list_clients_by_project(pid).unwrap();
                        // 所有客户都应关联到每个项目
                        for expected_client in &created_clients {
                            let found = clients_for_project.iter().find(|c| c.id == expected_client.id);
                            prop_assert!(
                                found.is_some(),
                                "重新打开后项目 {} 应仍关联客户 '{}'", pid, expected_client.name
                            );
                        }
                    }
                }
            }
        }
    }
}
