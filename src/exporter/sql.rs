use std::fs::File;
use std::io::Write;

use crate::error::{ExcelCliError, Result};
use crate::exporter::Exporter;
use crate::models::{CellValue, ExcelData};

/// SQL 方言
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SqlDialect {
    /// MySQL / MariaDB
    MySQL,
    /// PostgreSQL
    PostgreSQL,
    /// SQLite
    SQLite,
    /// Microsoft SQL Server
    SqlServer,
    /// Oracle
    Oracle,
}

/// SQL 语句模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SqlMode {
    /// INSERT 语句
    Insert,
    /// UPDATE 语句
    Update,
    /// UPSERT 语句（INSERT ... ON CONFLICT/DUPLICATE KEY UPDATE）
    Upsert,
}

impl SqlMode {
    /// 从字符串解析模式
    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "insert" => Ok(SqlMode::Insert),
            "update" => Ok(SqlMode::Update),
            "upsert" | "merge" => Ok(SqlMode::Upsert),
            _ => Err(ExcelCliError::UnsupportedFormat(format!(
                "不支持的 SQL 模式: {}",
                s
            ))),
        }
    }
}

impl SqlDialect {
    /// 从字符串解析方言
    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "mysql" | "mariadb" => Ok(SqlDialect::MySQL),
            "postgresql" | "postgres" | "pg" => Ok(SqlDialect::PostgreSQL),
            "sqlite" | "sqlite3" => Ok(SqlDialect::SQLite),
            "sqlserver" | "mssql" | "tsql" => Ok(SqlDialect::SqlServer),
            "oracle" => Ok(SqlDialect::Oracle),
            _ => Err(ExcelCliError::UnsupportedFormat(format!(
                "不支持的 SQL 方言: {}",
                s
            ))),
        }
    }

    /// 获取标识符引用符号
    pub fn quote_identifier(&self, identifier: &str) -> String {
        match self {
            SqlDialect::MySQL => format!("`{}`", identifier),
            SqlDialect::PostgreSQL => format!("\"{}\"", identifier),
            SqlDialect::SQLite => format!("\"{}\"", identifier),
            SqlDialect::SqlServer => format!("[{}]", identifier),
            SqlDialect::Oracle => format!("\"{}\"", identifier.to_uppercase()),
        }
    }

    /// 获取字符串值引用符号
    fn quote_string(&self, value: &str) -> String {
        // 转义单引号
        let escaped = value.replace('\'', "''");
        format!("'{}'", escaped)
    }

    /// 格式化值
    fn format_value(&self, value: &CellValue) -> String {
        match value {
            CellValue::String(s) => self.quote_string(s),
            CellValue::Number(n) => {
                // 检查是否为整数
                if n.fract() == 0.0 && n.is_finite() {
                    format!("{}", *n as i64)
                } else {
                    format!("{}", n)
                }
            }
            CellValue::Boolean(b) => match self {
                SqlDialect::MySQL | SqlDialect::SQLite => {
                    if *b {
                        "1".to_string()
                    } else {
                        "0".to_string()
                    }
                }
                SqlDialect::PostgreSQL => {
                    if *b {
                        "TRUE".to_string()
                    } else {
                        "FALSE".to_string()
                    }
                }
                SqlDialect::SqlServer | SqlDialect::Oracle => {
                    if *b {
                        "1".to_string()
                    } else {
                        "0".to_string()
                    }
                }
            },
            CellValue::Empty => "NULL".to_string(),
        }
    }
}

/// SQL 导出器
pub struct SqlExporter {
    dialect: SqlDialect,
    table_name: String,
    mode: SqlMode,
    primary_keys: Vec<String>,
    update_columns: Option<Vec<String>>,
    column_mapping: Option<Vec<String>>,
    batch_size: usize,
}

impl SqlExporter {
    /// 创建新的 SQL 导出器（默认INSERT模式）
    pub fn new(dialect: SqlDialect, table_name: String) -> Self {
        Self {
            dialect,
            table_name,
            mode: SqlMode::Insert,
            primary_keys: Vec::new(),
            update_columns: None,
            column_mapping: None,
            batch_size: 1000,
        }
    }

    /// 设置 SQL 模式
    pub fn with_mode(mut self, mode: SqlMode) -> Self {
        self.mode = mode;
        self
    }

    /// 设置主键列（用于 UPDATE 和 UPSERT）
    pub fn with_primary_keys(mut self, keys: Vec<String>) -> Self {
        self.primary_keys = keys;
        self
    }

    /// 设置要更新的列（用于 UPDATE 模式，如果为 None 则更新所有非主键列）
    pub fn with_update_columns(mut self, columns: Vec<String>) -> Self {
        self.update_columns = Some(columns);
        self
    }

    /// 设置列名映射
    pub fn with_column_mapping(mut self, mapping: Vec<String>) -> Self {
        self.column_mapping = Some(mapping);
        self
    }

    /// 设置批量大小
    pub fn with_batch_size(mut self, batch_size: usize) -> Self {
        self.batch_size = batch_size;
        self
    }

    /// 验证列名映射
    fn validate_column_mapping(&self, data: &ExcelData) -> Result<()> {
        if let Some(mapping) = &self.column_mapping {
            if mapping.len() != data.column_count() {
                return Err(ExcelCliError::ExportError(format!(
                    "列名映射数量({})与 Excel 列数({})不匹配",
                    mapping.len(),
                    data.column_count()
                )));
            }
        }
        Ok(())
    }

    /// 获取列名（使用映射或原始列名）
    fn get_column_names(&self, data: &ExcelData) -> Vec<String> {
        if let Some(mapping) = &self.column_mapping {
            mapping.clone()
        } else {
            data.headers.clone()
        }
    }

    /// 生成单条 INSERT 语句
    fn generate_single_insert(&self, table: &str, columns: &[String], row_values: &[String]) -> String {
        let quoted_columns: Vec<String> = columns
            .iter()
            .map(|col| self.dialect.quote_identifier(col))
            .collect();

        format!(
            "INSERT INTO {} ({}) VALUES ({});",
            self.dialect.quote_identifier(table),
            quoted_columns.join(", "),
            row_values.join(", ")
        )
    }

    /// 生成批量 INSERT 语句（MySQL 扩展语法）
    fn generate_batch_insert(
        &self,
        table: &str,
        columns: &[String],
        batch_values: &[Vec<String>],
    ) -> String {
        let quoted_columns: Vec<String> = columns
            .iter()
            .map(|col| self.dialect.quote_identifier(col))
            .collect();

        let values_clauses: Vec<String> = batch_values
            .iter()
            .map(|row| format!("({})", row.join(", ")))
            .collect();

        format!(
            "INSERT INTO {} ({}) VALUES\n{};",
            self.dialect.quote_identifier(table),
            quoted_columns.join(", "),
            values_clauses.join(",\n")
        )
    }

    /// 生成 UPDATE 语句
    fn generate_update(&self, table: &str, columns: &[String], row_values: &[String], _data: &ExcelData) -> Result<String> {
        if self.primary_keys.is_empty() {
            return Err(ExcelCliError::ExportError(
                "UPDATE 模式需要指定主键列（--primary-key）".to_string()
            ));
        }

        // 获取要更新的列（排除主键列）
        let update_cols: Vec<(String, String)> = columns
            .iter()
            .zip(row_values.iter())
            .filter(|(col, _)| !self.primary_keys.contains(col))
            .map(|(col, val)| (col.clone(), val.clone()))
            .collect();

        if update_cols.is_empty() {
            return Err(ExcelCliError::ExportError(
                "没有可更新的列（所有列都是主键）".to_string()
            ));
        }

        // 如果指定了update_columns，只更新这些列
        let filtered_cols: Vec<(String, String)> = if let Some(ref update_only) = self.update_columns {
            update_cols
                .into_iter()
                .filter(|(col, _)| update_only.contains(col))
                .collect()
        } else {
            update_cols
        };

        // 构建 SET 子句
        let set_clauses: Vec<String> = filtered_cols
            .iter()
            .map(|(col, val)| {
                format!("{} = {}", self.dialect.quote_identifier(col), val)
            })
            .collect();

        // 构建 WHERE 子句
        let where_clauses: Vec<String> = self.primary_keys
            .iter()
            .filter_map(|pk| {
                let idx = columns.iter().position(|c| c == pk)?;
                Some(format!(
                    "{} = {}",
                    self.dialect.quote_identifier(pk),
                    row_values.get(idx)?
                ))
            })
            .collect();

        Ok(format!(
            "UPDATE {} SET {} WHERE {};",
            self.dialect.quote_identifier(table),
            set_clauses.join(", "),
            where_clauses.join(" AND ")
        ))
    }

    /// 生成 UPSERT 语句（根据方言不同使用不同语法）
    fn generate_upsert(&self, table: &str, columns: &[String], row_values: &[String]) -> Result<String> {
        if self.primary_keys.is_empty() {
            return Err(ExcelCliError::ExportError(
                "UPSERT 模式需要指定主键列（--primary-key）".to_string()
            ));
        }

        let quoted_columns: Vec<String> = columns
            .iter()
            .map(|col| self.dialect.quote_identifier(col))
            .collect();

        let quoted_table = self.dialect.quote_identifier(table);

        match self.dialect {
            SqlDialect::MySQL => {
                // MySQL: INSERT ... ON DUPLICATE KEY UPDATE
                let update_clauses: Vec<String> = columns
                    .iter()
                    .filter(|col| !self.primary_keys.contains(col))
                    .map(|col| {
                        let quoted = self.dialect.quote_identifier(col);
                        format!("{} = VALUES({})", quoted, quoted)
                    })
                    .collect();

                Ok(format!(
                    "INSERT INTO {} ({}) VALUES ({}) ON DUPLICATE KEY UPDATE {};",
                    quoted_table,
                    quoted_columns.join(", "),
                    row_values.join(", "),
                    update_clauses.join(", ")
                ))
            }
            SqlDialect::PostgreSQL => {
                // PostgreSQL: INSERT ... ON CONFLICT ... DO UPDATE
                let conflict_columns: Vec<String> = self.primary_keys
                    .iter()
                    .map(|pk| self.dialect.quote_identifier(pk))
                    .collect();

                let update_clauses: Vec<String> = columns
                    .iter()
                    .filter(|col| !self.primary_keys.contains(col))
                    .map(|col| {
                        let quoted = self.dialect.quote_identifier(col);
                        format!("{} = EXCLUDED.{}", quoted, quoted)
                    })
                    .collect();

                Ok(format!(
                    "INSERT INTO {} ({}) VALUES ({}) ON CONFLICT ({}) DO UPDATE SET {};",
                    quoted_table,
                    quoted_columns.join(", "),
                    row_values.join(", "),
                    conflict_columns.join(", "),
                    update_clauses.join(", ")
                ))
            }
            SqlDialect::SQLite => {
                // SQLite: INSERT ... ON CONFLICT ... DO UPDATE
                let conflict_columns: Vec<String> = self.primary_keys
                    .iter()
                    .map(|pk| self.dialect.quote_identifier(pk))
                    .collect();

                let update_clauses: Vec<String> = columns
                    .iter()
                    .filter(|col| !self.primary_keys.contains(col))
                    .map(|col| {
                        let quoted = self.dialect.quote_identifier(col);
                        format!("{} = excluded.{}", quoted, quoted)
                    })
                    .collect();

                Ok(format!(
                    "INSERT INTO {} ({}) VALUES ({}) ON CONFLICT ({}) DO UPDATE SET {};",
                    quoted_table,
                    quoted_columns.join(", "),
                    row_values.join(", "),
                    conflict_columns.join(", "),
                    update_clauses.join(", ")
                ))
            }
            SqlDialect::SqlServer | SqlDialect::Oracle => {
                // SQL Server/Oracle: MERGE statement
                let source_values: Vec<String> = columns
                    .iter()
                    .zip(row_values.iter())
                    .map(|(col, val)| format!("{} AS {}", val, self.dialect.quote_identifier(col)))
                    .collect();

                let match_conditions: Vec<String> = self.primary_keys
                    .iter()
                    .map(|pk| {
                        let quoted = self.dialect.quote_identifier(pk);
                        format!("target.{} = source.{}", quoted, quoted)
                    })
                    .collect();

                let update_set: Vec<String> = columns
                    .iter()
                    .filter(|col| !self.primary_keys.contains(col))
                    .map(|col| {
                        let quoted = self.dialect.quote_identifier(col);
                        format!("target.{} = source.{}", quoted, quoted)
                    })
                    .collect();

                let insert_cols = quoted_columns.join(", ");
                let insert_vals: Vec<String> = columns
                    .iter()
                    .map(|col| format!("source.{}", self.dialect.quote_identifier(col)))
                    .collect();

                Ok(format!(
                    "MERGE INTO {} AS target USING (SELECT {}) AS source ON ({}) WHEN MATCHED THEN UPDATE SET {} WHEN NOT MATCHED THEN INSERT ({}) VALUES ({});",
                    quoted_table,
                    source_values.join(", "),
                    match_conditions.join(" AND "),
                    update_set.join(", "),
                    insert_cols,
                    insert_vals.join(", ")
                ))
            }
        }
    }
}

impl Exporter for SqlExporter {
    fn export(&self, data: &ExcelData, output_path: &str) -> Result<()> {
        // 验证列名映射
        self.validate_column_mapping(data)?;

        let mut file = File::create(output_path)?;

        // 获取列名
        let column_names = self.get_column_names(data);

        // 写入文件头注释
        writeln!(
            file,
            "-- Generated by excel-cli"
        )?;
        writeln!(
            file,
            "-- Dialect: {:?}",
            self.dialect
        )?;
        writeln!(
            file,
            "-- Mode: {:?}",
            self.mode
        )?;
        writeln!(
            file,
            "-- Table: {}",
            self.table_name
        )?;
        writeln!(
            file,
            "-- Rows: {}",
            data.row_count()
        )?;
        writeln!(file)?;

        // 根据模式导出
        match self.mode {
            SqlMode::Insert => {
                self.export_insert(data, &mut file, &column_names)?;
            }
            SqlMode::Update => {
                self.export_update(data, &mut file, &column_names)?;
            }
            SqlMode::Upsert => {
                self.export_upsert(data, &mut file, &column_names)?;
            }
        }

        file.flush()?;
        Ok(())
    }

    fn format_name(&self) -> &'static str {
        "SQL"
    }

    fn file_extension(&self) -> &'static str {
        "sql"
    }
}

impl SqlExporter {
    /// 导出 INSERT 语句
    fn export_insert(&self, data: &ExcelData, file: &mut File, column_names: &[String]) -> Result<()> {
        // 根据方言选择批量或单条插入
        let use_batch = matches!(
            self.dialect,
            SqlDialect::MySQL | SqlDialect::PostgreSQL | SqlDialect::SQLite
        ) && self.batch_size > 1;

        if use_batch {
            // 批量插入
            let mut batch_values = Vec::new();

            for (idx, row) in data.rows.iter().enumerate() {
                let mut row_values = Vec::new();

                for header in &data.headers {
                    let value = row.data.get(header).unwrap_or(&CellValue::Empty);
                    row_values.push(self.dialect.format_value(value));
                }

                batch_values.push(row_values);

                // 当达到批量大小或最后一行时，写入批量 INSERT
                if batch_values.len() >= self.batch_size || idx == data.rows.len() - 1 {
                    let sql = self.generate_batch_insert(
                        &self.table_name,
                        column_names,
                        &batch_values,
                    );
                    writeln!(file, "{}", sql)?;
                    writeln!(file)?;
                    batch_values.clear();
                }
            }
        } else {
            // 单条插入
            for row in &data.rows {
                let mut row_values = Vec::new();

                for header in &data.headers {
                    let value = row.data.get(header).unwrap_or(&CellValue::Empty);
                    row_values.push(self.dialect.format_value(value));
                }

                let sql = self.generate_single_insert(&self.table_name, column_names, &row_values);
                writeln!(file, "{}", sql)?;
            }
        }

        Ok(())
    }

    /// 导出 UPDATE 语句
    fn export_update(&self, data: &ExcelData, file: &mut File, column_names: &[String]) -> Result<()> {
        for row in &data.rows {
            let mut row_values = Vec::new();

            for header in &data.headers {
                let value = row.data.get(header).unwrap_or(&CellValue::Empty);
                row_values.push(self.dialect.format_value(value));
            }

            let sql = self.generate_update(&self.table_name, column_names, &row_values, data)?;
            writeln!(file, "{}", sql)?;
        }

        Ok(())
    }

    /// 导出 UPSERT 语句
    fn export_upsert(&self, data: &ExcelData, file: &mut File, column_names: &[String]) -> Result<()> {
        for row in &data.rows {
            let mut row_values = Vec::new();

            for header in &data.headers {
                let value = row.data.get(header).unwrap_or(&CellValue::Empty);
                row_values.push(self.dialect.format_value(value));
            }

            let sql = self.generate_upsert(&self.table_name, column_names, &row_values)?;
            writeln!(file, "{}", sql)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ExcelRow;
    use std::collections::HashMap;

    #[test]
    fn test_sql_dialect_quote() {
        assert_eq!(SqlDialect::MySQL.quote_identifier("table"), "`table`");
        assert_eq!(SqlDialect::PostgreSQL.quote_identifier("table"), "\"table\"");
        assert_eq!(SqlDialect::SQLite.quote_identifier("table"), "\"table\"");
        assert_eq!(SqlDialect::SqlServer.quote_identifier("table"), "[table]");
    }

    #[test]
    fn test_sql_export() {
        let mut data = ExcelData::new(
            "Sheet1".to_string(),
            vec!["Name".to_string(), "Age".to_string()],
        );

        let mut row1 = HashMap::new();
        row1.insert("Name".to_string(), CellValue::String("Alice".to_string()));
        row1.insert("Age".to_string(), CellValue::Number(30.0));
        data.add_row(ExcelRow { data: row1 });

        let exporter = SqlExporter::new(SqlDialect::MySQL, "users".to_string());
        let result = exporter.export(&data, "test_output.sql");
        assert!(result.is_ok());

        // 清理测试文件
        let _ = std::fs::remove_file("test_output.sql");
    }

    #[test]
    fn test_column_mapping_validation() {
        let data = ExcelData::new(
            "Sheet1".to_string(),
            vec!["Name".to_string(), "Age".to_string()],
        );

        let exporter = SqlExporter::new(SqlDialect::MySQL, "users".to_string())
            .with_column_mapping(vec!["user_name".to_string()]); // 故意设置错误数量

        let result = exporter.validate_column_mapping(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_format_values() {
        let dialect = SqlDialect::MySQL;

        assert_eq!(dialect.format_value(&CellValue::String("test".to_string())), "'test'");
        assert_eq!(dialect.format_value(&CellValue::Number(42.0)), "42");
        assert_eq!(dialect.format_value(&CellValue::Number(3.14)), "3.14");
        assert_eq!(dialect.format_value(&CellValue::Boolean(true)), "1");
        assert_eq!(dialect.format_value(&CellValue::Empty), "NULL");
    }
}
