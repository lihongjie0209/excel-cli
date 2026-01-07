use calamine::{open_workbook, Data, Reader, Xlsx};
use std::collections::HashMap;
use std::path::Path;

use crate::error::{ExcelCliError, Result};
use crate::models::{CellValue, ExcelData, ExcelRow};

/// Excel 读取器
pub struct ExcelReader {
    file_path: String,
}

impl ExcelReader {
    /// 创建新的 Excel 读取器
    pub fn new<P: AsRef<Path>>(file_path: P) -> Self {
        Self {
            file_path: file_path.as_ref().to_string_lossy().to_string(),
        }
    }

    /// 读取指定工作表的数据
    pub fn read_sheet(&self, sheet_name: Option<&str>) -> Result<ExcelData> {
        let mut workbook: Xlsx<_> = open_workbook(&self.file_path)
            .map_err(|e: calamine::XlsxError| ExcelCliError::ExcelReadError(e.to_string()))?;

        // 确定要读取的工作表名称
        let target_sheet = if let Some(name) = sheet_name {
            name.to_string()
        } else {
            // 如果未指定工作表，使用第一个工作表
            workbook
                .sheet_names()
                .first()
                .ok_or_else(|| ExcelCliError::ExcelReadError("工作簿中没有工作表".to_string()))?
                .clone()
        };

        // 读取工作表数据
        let range = workbook
            .worksheet_range(&target_sheet)
            .map_err(|e: calamine::XlsxError| ExcelCliError::ExcelReadError(e.to_string()))?;

        // 解析数据
        let mut rows_iter = range.rows();

        // 读取表头
        let headers = if let Some(header_row) = rows_iter.next() {
            header_row
                .iter()
                .enumerate()
                .map(|(idx, cell)| self.cell_to_string(cell, idx))
                .collect::<Vec<_>>()
        } else {
            return Err(ExcelCliError::ExcelReadError("工作表为空".to_string()));
        };

        let mut excel_data = ExcelData::new(target_sheet, headers.clone());

        // 读取数据行
        for row in rows_iter {
            let mut row_data = HashMap::new();

            for (idx, cell) in row.iter().enumerate() {
                if idx < headers.len() {
                    let header = &headers[idx];
                    let value = self.cell_to_value(cell);
                    row_data.insert(header.clone(), value);
                }
            }

            excel_data.add_row(ExcelRow { data: row_data });
        }

        Ok(excel_data)
    }

    /// 获取所有工作表名称
    pub fn get_sheet_names(&self) -> Result<Vec<String>> {
        let workbook: Xlsx<_> = open_workbook(&self.file_path)
            .map_err(|e: calamine::XlsxError| ExcelCliError::ExcelReadError(e.to_string()))?;

        Ok(workbook.sheet_names().to_vec())
    }

    /// 将单元格数据转换为字符串（用于表头）
    fn cell_to_string(&self, cell: &Data, col_index: usize) -> String {
        match cell {
            Data::Empty => format!("Column_{}", col_index + 1),
            Data::String(s) => s.clone(),
            Data::Float(f) => f.to_string(),
            Data::Int(i) => i.to_string(),
            Data::Bool(b) => b.to_string(),
            Data::Error(e) => format!("Error: {:?}", e),
            Data::DateTime(dt) => format!("{:?}", dt),
            Data::DateTimeIso(dt) => dt.clone(),
            Data::DurationIso(d) => d.clone(),
        }
    }

    /// 将单元格数据转换为 CellValue
    fn cell_to_value(&self, cell: &Data) -> CellValue {
        match cell {
            Data::Empty => CellValue::Empty,
            Data::String(s) => CellValue::String(s.clone()),
            Data::Float(f) => CellValue::Number(*f),
            Data::Int(i) => CellValue::Number(*i as f64),
            Data::Bool(b) => CellValue::Boolean(*b),
            Data::Error(e) => CellValue::String(format!("Error: {:?}", e)),
            Data::DateTime(dt) => CellValue::String(format!("{:?}", dt)),
            Data::DateTimeIso(dt) => CellValue::String(dt.clone()),
            Data::DurationIso(d) => CellValue::String(d.clone()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cell_to_value() {
        let reader = ExcelReader::new("test.xlsx");

        let empty = Data::Empty;
        assert!(matches!(reader.cell_to_value(&empty), CellValue::Empty));

        let string = Data::String("test".to_string());
        assert!(matches!(
            reader.cell_to_value(&string),
            CellValue::String(_)
        ));

        let number = Data::Float(42.0);
        assert!(matches!(
            reader.cell_to_value(&number),
            CellValue::Number(_)
        ));
    }
}
