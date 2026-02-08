// ============================================================================
// 统一错误类型定义
// 使用 thiserror 派生宏，遵循 Rust 错误处理最佳实践
// ============================================================================

use thiserror::Error;

/// 应用统一错误枚举
///
/// 覆盖所有业务场景的错误类型，每个变体对应一类错误。
/// 通过 `impl From<AppError> for String` 保持与现有 Tauri command 的兼容性
/// （Tauri command 要求返回 `Result<T, String>`）。
#[derive(Debug, Error)]
pub enum AppError {
    /// 参数验证失败（如客户名称为空、未选择模块）
    #[error("验证失败：{0}")]
    ValidationError(String),

    /// 构建过程中的错误（如文件复制、ZIP 打包失败）
    #[error("构建失败：{0}")]
    BuildError(String),

    /// 模块扫描失败
    #[error("模块扫描失败：{0}")]
    ScanError(String),

    /// 文件系统 IO 错误
    #[error("IO 错误：{0}")]
    IoError(#[from] std::io::Error),

    /// 数据库操作错误
    #[error("{0}")]
    DatabaseError(String),

    /// 不支持的技术栈类型
    #[error("不支持的技术栈类型：{0}")]
    UnsupportedTechStack(String),

    /// 用户取消操作（如关闭文件选择对话框）
    #[error("cancelled")]
    Cancelled,

    /// 系统文件管理器打开失败
    #[error("打开文件夹失败：{0}")]
    OpenFolderError(String),
}

/// 便捷类型别名，统一项目内的 Result 签名
pub type AppResult<T> = Result<T, AppError>;

/// 将 AppError 转换为 String，保持与 Tauri command 返回类型的兼容性
///
/// Tauri 的 `#[tauri::command]` 要求错误类型为 `String`，
/// 此转换确保所有错误都能正确传递到前端。
impl From<AppError> for String {
    fn from(err: AppError) -> Self {
        err.to_string()
    }
}
