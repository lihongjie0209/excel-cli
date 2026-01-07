use std::collections::HashMap;
use std::fs::File;
use std::io::Write;

use serde_json::Value as JsonValue;
use tera::{Context, Tera};

use crate::error::{ExcelCliError, Result};
use crate::exporter::Exporter;
use crate::models::{CellValue, ExcelData};

/// 内置模板类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuiltinTemplate {
    /// HTML 表格
    HtmlTable,
    /// Markdown 表格
    MarkdownTable,
    /// XML
    Xml,
    /// YAML
    Yaml,
}

impl BuiltinTemplate {
    /// 从字符串解析
    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "html" | "html-table" => Ok(BuiltinTemplate::HtmlTable),
            "markdown" | "md" | "md-table" => Ok(BuiltinTemplate::MarkdownTable),
            "xml" => Ok(BuiltinTemplate::Xml),
            "yaml" | "yml" => Ok(BuiltinTemplate::Yaml),
            _ => Err(ExcelCliError::UnsupportedFormat(format!(
                "不支持的内置模板: {}",
                s
            ))),
        }
    }

    /// 获取模板内容
    pub fn get_template(&self) -> &'static str {
        match self {
            BuiltinTemplate::HtmlTable => include_str!("../../templates/html_table.tera"),
            BuiltinTemplate::MarkdownTable => include_str!("../../templates/markdown_table.tera"),
            BuiltinTemplate::Xml => include_str!("../../templates/xml.tera"),
            BuiltinTemplate::Yaml => include_str!("../../templates/yaml.tera"),
        }
    }

    /// 获取文件扩展名
    pub fn file_extension(&self) -> &'static str {
        match self {
            BuiltinTemplate::HtmlTable => "html",
            BuiltinTemplate::MarkdownTable => "md",
            BuiltinTemplate::Xml => "xml",
            BuiltinTemplate::Yaml => "yaml",
        }
    }
}

/// 模板导出器
pub struct TemplateExporter {
    template_content: String,
    template_name: String,
    file_ext: String,
}

impl TemplateExporter {
    /// 使用自定义模板文件创建导出器
    pub fn from_file(template_path: &str) -> Result<Self> {
        let template_content = std::fs::read_to_string(template_path).map_err(|e| {
            ExcelCliError::ExportError(format!("无法读取模板文件 {}: {}", template_path, e))
        })?;

        let template_name = std::path::Path::new(template_path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("template")
            .to_string();

        let file_ext = std::path::Path::new(template_path)
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("txt")
            .to_string();

        Ok(Self {
            template_content,
            template_name,
            file_ext,
        })
    }

    /// 使用内置模板创建导出器
    pub fn from_builtin(builtin: BuiltinTemplate) -> Self {
        Self {
            template_content: builtin.get_template().to_string(),
            template_name: format!("{:?}", builtin),
            file_ext: builtin.file_extension().to_string(),
        }
    }

    /// 将 ExcelData 转换为 Tera Context
    fn create_context(&self, data: &ExcelData) -> Context {
        let mut context = Context::new();

        // 添加工作表名称
        context.insert("sheet_name", &data.sheet_name);

        // 添加表头
        context.insert("headers", &data.headers);

        // 转换行数据为 Vec<HashMap<String, Value>>
        let rows: Vec<HashMap<String, JsonValue>> = data
            .rows
            .iter()
            .map(|row| {
                let mut row_map = HashMap::new();
                for (key, value) in &row.data {
                    let json_value = match value {
                        CellValue::String(s) => JsonValue::String(s.clone()),
                        CellValue::Number(n) => {
                            if n.fract() == 0.0 && n.is_finite() {
                                JsonValue::Number((*n as i64).into())
                            } else {
                                JsonValue::Number(
                                    serde_json::Number::from_f64(*n)
                                        .unwrap_or_else(|| 0.into()),
                                )
                            }
                        }
                        CellValue::Boolean(b) => JsonValue::Bool(*b),
                        CellValue::Empty => JsonValue::Null,
                    };
                    row_map.insert(key.clone(), json_value);
                }
                row_map
            })
            .collect();

        context.insert("rows", &rows);
        context.insert("row_count", &data.row_count());
        context.insert("column_count", &data.column_count());

        context
    }
}

impl Exporter for TemplateExporter {
    fn export(&self, data: &ExcelData, output_path: &str) -> Result<()> {
        // 创建 Tera 实例
        let mut tera = Tera::default();

        // 添加模板
        tera.add_raw_template(&self.template_name, &self.template_content)
            .map_err(|e| {
                ExcelCliError::ExportError(format!("模板解析失败: {}", e))
            })?;

        // 创建上下文
        let context = self.create_context(data);

        // 渲染模板
        let rendered = tera
            .render(&self.template_name, &context)
            .map_err(|e| ExcelCliError::ExportError(format!("模板渲染失败: {}", e)))?;

        // 写入文件
        let mut file = File::create(output_path)?;
        file.write_all(rendered.as_bytes())?;
        file.flush()?;

        Ok(())
    }

    fn format_name(&self) -> &'static str {
        "Template"
    }

    fn file_extension(&self) -> &'static str {
        // 根据实际扩展名返回对应的静态字符串
        match self.file_ext.as_str() {
            "html" => "html",
            "md" | "markdown" => "md",
            "xml" => "xml",
            "yaml" | "yml" => "yaml",
            _ => "txt",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ExcelRow;

    #[test]
    fn test_builtin_template_from_str() {
        assert_eq!(
            BuiltinTemplate::from_str("html").unwrap(),
            BuiltinTemplate::HtmlTable
        );
        assert_eq!(
            BuiltinTemplate::from_str("markdown").unwrap(),
            BuiltinTemplate::MarkdownTable
        );
        assert_eq!(
            BuiltinTemplate::from_str("xml").unwrap(),
            BuiltinTemplate::Xml
        );
    }

    #[test]
    fn test_context_creation() {
        let data = ExcelData {
            sheet_name: "Test".to_string(),
            headers: vec!["Name".to_string(), "Age".to_string()],
            rows: vec![
                ExcelRow::from_vec(vec![
                    CellValue::String("Alice".to_string()),
                    CellValue::Number(30.0),
                ]),
                ExcelRow::from_vec(vec![
                    CellValue::String("Bob".to_string()),
                    CellValue::Number(25.0),
                ]),
            ],
        };

        let exporter = TemplateExporter::from_builtin(BuiltinTemplate::HtmlTable);
        let context = exporter.create_context(&data);

        // Verify context contains expected data
        assert!(context.get("sheet_name").is_some());
        assert!(context.get("headers").is_some());
        assert!(context.get("rows").is_some());
    }
}
