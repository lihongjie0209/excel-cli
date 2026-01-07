use std::fs::File;

use crate::error::Result;
use crate::exporter::Exporter;
use crate::models::ExcelData;

/// CSV 导出器
pub struct CsvExporter {
    delimiter: u8,
}

impl CsvExporter {
    /// 创建新的 CSV 导出器
    pub fn new() -> Self {
        Self { delimiter: b',' }
    }

    /// 设置分隔符
    pub fn with_delimiter(mut self, delimiter: u8) -> Self {
        self.delimiter = delimiter;
        self
    }
}

impl Default for CsvExporter {
    fn default() -> Self {
        Self::new()
    }
}

impl Exporter for CsvExporter {
    fn export(&self, data: &ExcelData, output_path: &str) -> Result<()> {
        let file = File::create(output_path)?;
        let mut writer = csv::WriterBuilder::new()
            .delimiter(self.delimiter)
            .from_writer(file);

        // 写入表头
        writer.write_record(&data.headers)?;

        // 写入数据行
        for row in &data.rows {
            let mut record = Vec::new();
            for header in &data.headers {
                let value = row
                    .data
                    .get(header)
                    .map(|v| v.to_string())
                    .unwrap_or_default();
                record.push(value);
            }
            writer.write_record(&record)?;
        }

        writer.flush()?;
        Ok(())
    }

    fn format_name(&self) -> &'static str {
        "CSV"
    }

    fn file_extension(&self) -> &'static str {
        "csv"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{CellValue, ExcelRow};
    use std::collections::HashMap;

    #[test]
    fn test_csv_export() {
        let mut data = ExcelData::new(
            "Sheet1".to_string(),
            vec!["Name".to_string(), "Age".to_string()],
        );

        let mut row1 = HashMap::new();
        row1.insert("Name".to_string(), CellValue::String("Alice".to_string()));
        row1.insert("Age".to_string(), CellValue::Number(30.0));
        data.add_row(ExcelRow { data: row1 });

        let exporter = CsvExporter::new();
        let result = exporter.export(&data, "test_output.csv");
        assert!(result.is_ok());

        // 清理测试文件
        let _ = std::fs::remove_file("test_output.csv");
    }
}
