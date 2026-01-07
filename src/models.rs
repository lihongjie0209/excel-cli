use serde::{Deserialize, Serialize};

/// Excel 行数据表示
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExcelRow {
    /// 行数据，使用 Map 存储列名到值的映射
    #[serde(flatten)]
    pub data: std::collections::HashMap<String, CellValue>,
}

/// 单元格值类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CellValue {
    /// 字符串值
    String(String),
    /// 数值
    Number(f64),
    /// 布尔值
    Boolean(bool),
    /// 空值
    Empty,
}

impl CellValue {
    /// 转换为字符串
    pub fn to_string(&self) -> String {
        match self {
            CellValue::String(s) => s.clone(),
            CellValue::Number(n) => n.to_string(),
            CellValue::Boolean(b) => b.to_string(),
            CellValue::Empty => String::new(),
        }
    }

    /// 判断是否为空
    pub fn is_empty(&self) -> bool {
        matches!(self, CellValue::Empty)
    }
}

/// Excel 表格数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExcelData {
    /// 工作表名称
    pub sheet_name: String,
    /// 表头（列名）
    pub headers: Vec<String>,
    /// 行数据
    pub rows: Vec<ExcelRow>,
}

impl ExcelData {
    /// 创建新的 Excel 数据
    pub fn new(sheet_name: String, headers: Vec<String>) -> Self {
        Self {
            sheet_name,
            headers,
            rows: Vec::new(),
        }
    }

    /// 添加行数据
    pub fn add_row(&mut self, row: ExcelRow) {
        self.rows.push(row);
    }

    /// 获取行数
    pub fn row_count(&self) -> usize {
        self.rows.len()
    }

    /// 获取列数
    pub fn column_count(&self) -> usize {
        self.headers.len()
    }
}

impl ExcelRow {
    /// 从值的向量创建行（用于测试）
    #[cfg(test)]
    pub fn from_vec(values: Vec<CellValue>) -> Self {
        use std::collections::HashMap;
        let mut data = HashMap::new();
        for (i, value) in values.into_iter().enumerate() {
            data.insert(format!("col{}", i), value);
        }
        Self { data }
    }
}
