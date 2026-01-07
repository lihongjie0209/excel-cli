pub mod json;
pub mod csv;
pub mod sql;
pub mod template;

use crate::error::Result;
use crate::models::ExcelData;

pub use sql::{SqlDialect, SqlExporter, SqlMode};
pub use template::{BuiltinTemplate, TemplateExporter};

/// 导出器 trait，用于定义导出接口
/// 
/// 实现此 trait 可以添加新的导出格式
pub trait Exporter {
    /// 导出数据到指定路径
    /// 
    /// # 参数
    /// * `data` - Excel 数据
    /// * `output_path` - 输出文件路径
    fn export(&self, data: &ExcelData, output_path: &str) -> Result<()>;

    /// 获取导出格式名称
    fn format_name(&self) -> &'static str;

    /// 获取默认文件扩展名
    fn file_extension(&self) -> &'static str;
}

/// 导出器配置
pub struct ExporterConfig {
    /// SQL 方言（仅用于 SQL 格式）
    pub sql_dialect: Option<String>,
    /// SQL 表名（仅用于 SQL 格式）
    pub sql_table: Option<String>,
    /// SQL 模式（仅用于 SQL 格式）
    pub sql_mode: Option<String>,
    /// 主键列（仅用于 SQL UPDATE/UPSERT 模式）
    pub primary_keys: Option<Vec<String>>,
    /// 更新列（仅用于 SQL UPDATE 模式）
    pub update_columns: Option<Vec<String>>,
    /// 列名映射（仅用于 SQL 格式）
    pub column_mapping: Option<Vec<String>>,
    /// 自定义模板文件路径（仅用于 template 格式）
    pub template_path: Option<String>,
}

impl Default for ExporterConfig {
    fn default() -> Self {
        Self {
            sql_dialect: None,
            sql_table: None,
            sql_mode: None,
            primary_keys: None,
            update_columns: None,
            column_mapping: None,
            template_path: None,
        }
    }
}

/// 导出器工厂
pub struct ExporterFactory;

impl ExporterFactory {
    /// 根据格式名称和配置创建导出器
    pub fn create(format: &str, config: ExporterConfig) -> Result<Box<dyn Exporter>> {
        match format.to_lowercase().as_str() {
            "json" => Ok(Box::new(json::JsonExporter::new())),
            "csv" => Ok(Box::new(csv::CsvExporter::new())),
            "sql" => {
                let dialect_str = config.sql_dialect.as_deref().unwrap_or("mysql");
                let dialect = SqlDialect::from_str(dialect_str)?;
                let table_name = config.sql_table.unwrap_or_else(|| "table_name".to_string());
                
                let mut exporter = SqlExporter::new(dialect, table_name);
                
                // 设置 SQL 模式
                if let Some(mode_str) = config.sql_mode {
                    let mode = SqlMode::from_str(&mode_str)?;
                    exporter = exporter.with_mode(mode);
                }
                
                // 设置主键
                if let Some(keys) = config.primary_keys {
                    exporter = exporter.with_primary_keys(keys);
                }
                
                // 设置更新列
                if let Some(cols) = config.update_columns {
                    exporter = exporter.with_update_columns(cols);
                }
                
                // 设置列名映射
                if let Some(mapping) = config.column_mapping {
                    exporter = exporter.with_column_mapping(mapping);
                }
                
                Ok(Box::new(exporter))
            }
            "template" => {
                if let Some(template_path) = config.template_path {
                    Ok(Box::new(template::TemplateExporter::from_file(&template_path)?))
                } else {
                    Err(crate::error::ExcelCliError::ExportError(
                        "Template 格式需要指定 --template 参数".to_string(),
                    ))
                }
            }
            "html" | "html-table" => {
                Ok(Box::new(template::TemplateExporter::from_builtin(
                    template::BuiltinTemplate::HtmlTable,
                )))
            }
            "markdown" | "md" | "md-table" => {
                Ok(Box::new(template::TemplateExporter::from_builtin(
                    template::BuiltinTemplate::MarkdownTable,
                )))
            }
            "xml" => {
                Ok(Box::new(template::TemplateExporter::from_builtin(
                    template::BuiltinTemplate::Xml,
                )))
            }
            "yaml" | "yml" => {
                Ok(Box::new(template::TemplateExporter::from_builtin(
                    template::BuiltinTemplate::Yaml,
                )))
            }
            _ => Err(crate::error::ExcelCliError::UnsupportedFormat(
                format.to_string(),
            )),
        }
    }

    /// 获取所有支持的格式
    pub fn supported_formats() -> Vec<&'static str> {
        vec!["json", "csv", "sql", "template", "html", "markdown", "xml", "yaml"]
    }
}
