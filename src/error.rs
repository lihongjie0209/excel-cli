use thiserror::Error;

/// 应用程序错误类型
#[derive(Error, Debug)]
pub enum ExcelCliError {
    /// Excel 文件读取错误
    #[error("无法读取 Excel 文件: {0}")]
    ExcelReadError(String),

    /// 工作表不存在
    #[error("工作表 '{0}' 不存在")]
    SheetNotFound(String),

    /// 导出错误
    #[error("导出失败: {0}")]
    ExportError(String),

    /// 文件 I/O 错误
    #[error("文件操作失败: {0}")]
    IoError(#[from] std::io::Error),

    /// JSON 序列化错误
    #[error("JSON 序列化失败: {0}")]
    JsonError(#[from] serde_json::Error),

    /// CSV 错误
    #[error("CSV 处理失败: {0}")]
    CsvError(#[from] csv::Error),

    /// Calamine 错误
    #[error("Excel 处理失败: {0}")]
    CalamineError(#[from] calamine::Error),

    /// 不支持的格式
    #[error("不支持的导出格式: {0}")]
    UnsupportedFormat(String),
}

/// 应用程序结果类型
pub type Result<T> = std::result::Result<T, ExcelCliError>;
