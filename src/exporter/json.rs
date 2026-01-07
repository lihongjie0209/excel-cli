use std::fs::File;
use std::io::Write;

use crate::error::Result;
use crate::exporter::Exporter;
use crate::models::ExcelData;

/// JSON 导出器
pub struct JsonExporter {
    pretty: bool,
}

impl JsonExporter {
    /// 创建新的 JSON 导出器
    pub fn new() -> Self {
        Self { pretty: true }
    }

    /// 设置是否使用格式化输出
    pub fn with_pretty(mut self, pretty: bool) -> Self {
        self.pretty = pretty;
        self
    }
}

impl Default for JsonExporter {
    fn default() -> Self {
        Self::new()
    }
}

impl Exporter for JsonExporter {
    fn export(&self, data: &ExcelData, output_path: &str) -> Result<()> {
        let json_string = if self.pretty {
            serde_json::to_string_pretty(&data.rows)?
        } else {
            serde_json::to_string(&data.rows)?
        };

        let mut file = File::create(output_path)?;
        file.write_all(json_string.as_bytes())?;

        Ok(())
    }

    fn format_name(&self) -> &'static str {
        "JSON"
    }

    fn file_extension(&self) -> &'static str {
        "json"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{CellValue, ExcelRow};
    use std::collections::HashMap;

    #[test]
    fn test_json_export() {
        let mut data = ExcelData::new(
            "Sheet1".to_string(),
            vec!["Name".to_string(), "Age".to_string()],
        );

        let mut row1 = HashMap::new();
        row1.insert("Name".to_string(), CellValue::String("Alice".to_string()));
        row1.insert("Age".to_string(), CellValue::Number(30.0));
        data.add_row(ExcelRow { data: row1 });

        let exporter = JsonExporter::new();
        let result = exporter.export(&data, "test_output.json");
        assert!(result.is_ok());

        // 清理测试文件
        let _ = std::fs::remove_file("test_output.json");
    }
}
