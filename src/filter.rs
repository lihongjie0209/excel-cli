use crate::error::{ExcelCliError, Result};
use crate::models::{CellValue, ExcelData, ExcelRow};
use std::collections::HashMap;

/// 数据过滤器
pub struct DataFilter {
    /// 选择的列名（None 表示选择所有列）
    select_columns: Option<Vec<String>>,
    /// 排除的列名
    exclude_columns: Vec<String>,
    /// 简单条件过滤（列名 -> 条件）
    conditions: Vec<FilterCondition>,
}

/// 过滤条件
#[derive(Debug, Clone)]
pub struct FilterCondition {
    pub column: String,
    pub operator: FilterOperator,
    pub value: String,
}

/// 过滤操作符
#[derive(Debug, Clone, PartialEq)]
pub enum FilterOperator {
    /// 等于
    Equal,
    /// 不等于
    NotEqual,
    /// 大于
    GreaterThan,
    /// 小于
    LessThan,
    /// 大于等于
    GreaterThanOrEqual,
    /// 小于等于
    LessThanOrEqual,
    /// 包含（字符串）
    Contains,
    /// 不包含（字符串）
    NotContains,
    /// 为空
    IsEmpty,
    /// 不为空
    IsNotEmpty,
}

impl FilterOperator {
    /// 从字符串解析操作符
    pub fn from_str(s: &str) -> Result<Self> {
        match s {
            "==" | "=" => Ok(FilterOperator::Equal),
            "!=" | "<>" => Ok(FilterOperator::NotEqual),
            ">" => Ok(FilterOperator::GreaterThan),
            "<" => Ok(FilterOperator::LessThan),
            ">=" => Ok(FilterOperator::GreaterThanOrEqual),
            "<=" => Ok(FilterOperator::LessThanOrEqual),
            "contains" | "CONTAINS" => Ok(FilterOperator::Contains),
            "not_contains" | "NOT_CONTAINS" => Ok(FilterOperator::NotContains),
            "is_empty" | "IS_EMPTY" => Ok(FilterOperator::IsEmpty),
            "is_not_empty" | "IS_NOT_EMPTY" => Ok(FilterOperator::IsNotEmpty),
            _ => Err(ExcelCliError::ExportError(format!(
                "不支持的过滤操作符: {}",
                s
            ))),
        }
    }

    /// 应用条件
    fn apply(&self, cell_value: &CellValue, filter_value: &str) -> bool {
        match self {
            FilterOperator::IsEmpty => cell_value.is_empty(),
            FilterOperator::IsNotEmpty => !cell_value.is_empty(),
            FilterOperator::Equal => {
                let cell_str = cell_value.to_string();
                cell_str == filter_value
            }
            FilterOperator::NotEqual => {
                let cell_str = cell_value.to_string();
                cell_str != filter_value
            }
            FilterOperator::GreaterThan => {
                if let CellValue::Number(n) = cell_value {
                    if let Ok(filter_num) = filter_value.parse::<f64>() {
                        return *n > filter_num;
                    }
                }
                false
            }
            FilterOperator::LessThan => {
                if let CellValue::Number(n) = cell_value {
                    if let Ok(filter_num) = filter_value.parse::<f64>() {
                        return *n < filter_num;
                    }
                }
                false
            }
            FilterOperator::GreaterThanOrEqual => {
                if let CellValue::Number(n) = cell_value {
                    if let Ok(filter_num) = filter_value.parse::<f64>() {
                        return *n >= filter_num;
                    }
                }
                false
            }
            FilterOperator::LessThanOrEqual => {
                if let CellValue::Number(n) = cell_value {
                    if let Ok(filter_num) = filter_value.parse::<f64>() {
                        return *n <= filter_num;
                    }
                }
                false
            }
            FilterOperator::Contains => {
                let cell_str = cell_value.to_string().to_lowercase();
                let filter_str = filter_value.to_lowercase();
                cell_str.contains(&filter_str)
            }
            FilterOperator::NotContains => {
                let cell_str = cell_value.to_string().to_lowercase();
                let filter_str = filter_value.to_lowercase();
                !cell_str.contains(&filter_str)
            }
        }
    }
}

impl FilterCondition {
    /// 解析过滤条件字符串
    /// 格式: "column operator value" 或 "column operator" (对于 is_empty/is_not_empty)
    pub fn parse(condition_str: &str) -> Result<Self> {
        let parts: Vec<&str> = condition_str.trim().split_whitespace().collect();

        if parts.len() < 2 {
            return Err(ExcelCliError::ExportError(format!(
                "无效的过滤条件: {}",
                condition_str
            )));
        }

        let column = parts[0].to_string();
        let operator = FilterOperator::from_str(parts[1])?;

        // is_empty 和 is_not_empty 不需要值
        let value = if matches!(operator, FilterOperator::IsEmpty | FilterOperator::IsNotEmpty) {
            String::new()
        } else if parts.len() < 3 {
            return Err(ExcelCliError::ExportError(format!(
                "过滤条件缺少值: {}",
                condition_str
            )));
        } else {
            // 支持带引号的值
            let value_part = parts[2..].join(" ");
            value_part.trim_matches(|c| c == '"' || c == '\'').to_string()
        };

        Ok(FilterCondition {
            column,
            operator,
            value,
        })
    }

    /// 检查行是否满足条件
    fn matches(&self, row: &ExcelRow) -> bool {
        if let Some(cell_value) = row.data.get(&self.column) {
            self.operator.apply(cell_value, &self.value)
        } else {
            false
        }
    }
}

impl DataFilter {
    /// 创建新的数据过滤器
    pub fn new() -> Self {
        Self {
            select_columns: None,
            exclude_columns: Vec::new(),
            conditions: Vec::new(),
        }
    }

    /// 设置选择的列
    pub fn with_select(mut self, columns: Vec<String>) -> Self {
        self.select_columns = Some(columns);
        self
    }

    /// 设置排除的列
    pub fn with_exclude(mut self, columns: Vec<String>) -> Self {
        self.exclude_columns = columns;
        self
    }

    /// 添加过滤条件
    pub fn with_condition(mut self, condition: FilterCondition) -> Self {
        self.conditions.push(condition);
        self
    }

    /// 应用过滤到数据
    pub fn apply(&self, data: &ExcelData) -> Result<ExcelData> {
        // 确定最终的列
        let final_columns = self.determine_columns(&data.headers)?;

        // 创建新的数据集
        let mut filtered_data = ExcelData::new(data.sheet_name.clone(), final_columns.clone());

        // 过滤行
        for row in &data.rows {
            // 检查所有条件
            let mut matches = true;
            for condition in &self.conditions {
                if !condition.matches(row) {
                    matches = false;
                    break;
                }
            }

            if matches {
                // 创建新行，只包含选定的列
                let mut new_row_data = HashMap::new();
                for col in &final_columns {
                    if let Some(value) = row.data.get(col) {
                        new_row_data.insert(col.clone(), value.clone());
                    }
                }
                filtered_data.add_row(ExcelRow { data: new_row_data });
            }
        }

        Ok(filtered_data)
    }

    /// 确定最终的列列表
    fn determine_columns(&self, original_headers: &[String]) -> Result<Vec<String>> {
        if let Some(ref select_cols) = self.select_columns {
            // 如果指定了 select，只使用这些列
            // 验证所有选择的列都存在
            for col in select_cols {
                if !original_headers.contains(col) {
                    return Err(ExcelCliError::ExportError(format!(
                        "列 '{}' 不存在于 Excel 数据中",
                        col
                    )));
                }
            }
            Ok(select_cols.clone())
        } else {
            // 否则使用所有列，但排除指定的列
            let result: Vec<String> = original_headers
                .iter()
                .filter(|col| !self.exclude_columns.contains(col))
                .cloned()
                .collect();

            if result.is_empty() {
                return Err(ExcelCliError::ExportError(
                    "过滤后没有剩余的列".to_string(),
                ));
            }

            Ok(result)
        }
    }
}

impl Default for DataFilter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_condition() {
        let cond = FilterCondition::parse("Age > 30").unwrap();
        assert_eq!(cond.column, "Age");
        assert_eq!(cond.operator, FilterOperator::GreaterThan);
        assert_eq!(cond.value, "30");

        let cond2 = FilterCondition::parse("Name == Alice").unwrap();
        assert_eq!(cond2.column, "Name");
        assert_eq!(cond2.operator, FilterOperator::Equal);
        assert_eq!(cond2.value, "Alice");

        let cond3 = FilterCondition::parse("City contains 北京").unwrap();
        assert_eq!(cond3.column, "City");
        assert_eq!(cond3.operator, FilterOperator::Contains);
        assert_eq!(cond3.value, "北京");
    }

    #[test]
    fn test_filter_conditions() {
        let mut data = ExcelData::new(
            "Sheet1".to_string(),
            vec!["Name".to_string(), "Age".to_string()],
        );

        let mut row1 = HashMap::new();
        row1.insert("Name".to_string(), CellValue::String("Alice".to_string()));
        row1.insert("Age".to_string(), CellValue::Number(30.0));
        data.add_row(ExcelRow { data: row1 });

        let mut row2 = HashMap::new();
        row2.insert("Name".to_string(), CellValue::String("Bob".to_string()));
        row2.insert("Age".to_string(), CellValue::Number(25.0));
        data.add_row(ExcelRow { data: row2 });

        // 过滤 Age > 27
        let filter = DataFilter::new()
            .with_condition(FilterCondition::parse("Age > 27").unwrap());

        let filtered = filter.apply(&data).unwrap();
        assert_eq!(filtered.row_count(), 1);
    }

    #[test]
    fn test_column_selection() {
        let mut data = ExcelData::new(
            "Sheet1".to_string(),
            vec!["Name".to_string(), "Age".to_string(), "City".to_string()],
        );

        let mut row1 = HashMap::new();
        row1.insert("Name".to_string(), CellValue::String("Alice".to_string()));
        row1.insert("Age".to_string(), CellValue::Number(30.0));
        row1.insert("City".to_string(), CellValue::String("Beijing".to_string()));
        data.add_row(ExcelRow { data: row1 });

        // 只选择 Name 和 Age
        let filter = DataFilter::new()
            .with_select(vec!["Name".to_string(), "Age".to_string()]);

        let filtered = filter.apply(&data).unwrap();
        assert_eq!(filtered.column_count(), 2);
        assert_eq!(filtered.headers, vec!["Name", "Age"]);
    }
}
