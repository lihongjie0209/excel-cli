use crate::error::Result;
use crate::exporter::sql::SqlDialect;
use crate::models::{CellValue, ExcelData};

/// 数据类型推断器
pub struct TypeInference;

/// SQL 数据类型
#[derive(Debug, Clone, PartialEq)]
pub enum SqlType {
    /// 整数
    Integer,
    /// 大整数
    BigInt,
    /// 浮点数
    Float,
    /// 双精度浮点
    Double,
    /// 布尔
    Boolean,
    /// 可变字符串
    Varchar(usize),
    /// 文本
    Text,
    /// 日期时间
    DateTime,
    /// 日期
    Date,
}

impl SqlType {
    /// 转换为 SQL 类型字符串（根据方言）
    pub fn to_sql_string(&self, dialect: &SqlDialect) -> String {
        match (self, dialect) {
            (SqlType::Integer, SqlDialect::MySQL) => "INT".to_string(),
            (SqlType::Integer, SqlDialect::PostgreSQL) => "INTEGER".to_string(),
            (SqlType::Integer, SqlDialect::SQLite) => "INTEGER".to_string(),
            (SqlType::Integer, SqlDialect::SqlServer) => "INT".to_string(),
            (SqlType::Integer, SqlDialect::Oracle) => "NUMBER(10)".to_string(),

            (SqlType::BigInt, SqlDialect::MySQL) => "BIGINT".to_string(),
            (SqlType::BigInt, SqlDialect::PostgreSQL) => "BIGINT".to_string(),
            (SqlType::BigInt, SqlDialect::SQLite) => "INTEGER".to_string(),
            (SqlType::BigInt, SqlDialect::SqlServer) => "BIGINT".to_string(),
            (SqlType::BigInt, SqlDialect::Oracle) => "NUMBER(19)".to_string(),

            (SqlType::Float, SqlDialect::MySQL) => "FLOAT".to_string(),
            (SqlType::Float, SqlDialect::PostgreSQL) => "REAL".to_string(),
            (SqlType::Float, SqlDialect::SQLite) => "REAL".to_string(),
            (SqlType::Float, SqlDialect::SqlServer) => "FLOAT".to_string(),
            (SqlType::Float, SqlDialect::Oracle) => "BINARY_FLOAT".to_string(),

            (SqlType::Double, SqlDialect::MySQL) => "DOUBLE".to_string(),
            (SqlType::Double, SqlDialect::PostgreSQL) => "DOUBLE PRECISION".to_string(),
            (SqlType::Double, SqlDialect::SQLite) => "REAL".to_string(),
            (SqlType::Double, SqlDialect::SqlServer) => "FLOAT".to_string(),
            (SqlType::Double, SqlDialect::Oracle) => "BINARY_DOUBLE".to_string(),

            (SqlType::Boolean, SqlDialect::MySQL) => "BOOLEAN".to_string(),
            (SqlType::Boolean, SqlDialect::PostgreSQL) => "BOOLEAN".to_string(),
            (SqlType::Boolean, SqlDialect::SQLite) => "INTEGER".to_string(),
            (SqlType::Boolean, SqlDialect::SqlServer) => "BIT".to_string(),
            (SqlType::Boolean, SqlDialect::Oracle) => "NUMBER(1)".to_string(),

            (SqlType::Varchar(len), SqlDialect::MySQL) => format!("VARCHAR({})", len),
            (SqlType::Varchar(len), SqlDialect::PostgreSQL) => format!("VARCHAR({})", len),
            (SqlType::Varchar(_len), SqlDialect::SQLite) => "TEXT".to_string(),
            (SqlType::Varchar(len), SqlDialect::SqlServer) => format!("VARCHAR({})", len),
            (SqlType::Varchar(len), SqlDialect::Oracle) => format!("VARCHAR2({})", len),

            (SqlType::Text, SqlDialect::MySQL) => "TEXT".to_string(),
            (SqlType::Text, SqlDialect::PostgreSQL) => "TEXT".to_string(),
            (SqlType::Text, SqlDialect::SQLite) => "TEXT".to_string(),
            (SqlType::Text, SqlDialect::SqlServer) => "NVARCHAR(MAX)".to_string(),
            (SqlType::Text, SqlDialect::Oracle) => "CLOB".to_string(),

            (SqlType::DateTime, SqlDialect::MySQL) => "DATETIME".to_string(),
            (SqlType::DateTime, SqlDialect::PostgreSQL) => "TIMESTAMP".to_string(),
            (SqlType::DateTime, SqlDialect::SQLite) => "TEXT".to_string(),
            (SqlType::DateTime, SqlDialect::SqlServer) => "DATETIME2".to_string(),
            (SqlType::DateTime, SqlDialect::Oracle) => "TIMESTAMP".to_string(),

            (SqlType::Date, SqlDialect::MySQL) => "DATE".to_string(),
            (SqlType::Date, SqlDialect::PostgreSQL) => "DATE".to_string(),
            (SqlType::Date, SqlDialect::SQLite) => "TEXT".to_string(),
            (SqlType::Date, SqlDialect::SqlServer) => "DATE".to_string(),
            (SqlType::Date, SqlDialect::Oracle) => "DATE".to_string(),
        }
    }
}

impl TypeInference {
    /// 推断列的 SQL 类型
    pub fn infer_column_type(data: &ExcelData, column: &str) -> SqlType {
        let mut has_number = false;
        let mut has_float = false;
        let mut has_boolean = false;
        let mut has_string = false;
        let mut max_string_len = 0;
        let mut all_empty = true;

        for row in &data.rows {
            if let Some(value) = row.data.get(column) {
                match value {
                    CellValue::Number(n) => {
                        all_empty = false;
                        has_number = true;
                        if n.fract() != 0.0 {
                            has_float = true;
                        }
                    }
                    CellValue::Boolean(_) => {
                        all_empty = false;
                        has_boolean = true;
                    }
                    CellValue::String(s) => {
                        all_empty = false;
                        has_string = true;
                        max_string_len = max_string_len.max(s.len());
                    }
                    CellValue::Empty => {}
                }
            }
        }

        // 如果全是空值，默认为 VARCHAR
        if all_empty {
            return SqlType::Varchar(255);
        }

        // 类型优先级判断
        if has_boolean && !has_number && !has_string {
            SqlType::Boolean
        } else if has_number && !has_string {
            if has_float {
                SqlType::Double
            } else {
                SqlType::Integer
            }
        } else if has_string {
            // 根据最大长度决定使用 VARCHAR 还是 TEXT
            if max_string_len <= 255 {
                SqlType::Varchar((max_string_len + 50).min(255))
            } else if max_string_len <= 1000 {
                SqlType::Varchar(1000)
            } else {
                SqlType::Text
            }
        } else {
            SqlType::Varchar(255)
        }
    }
}

/// CREATE TABLE 语句生成器
pub struct SchemaGenerator {
    dialect: SqlDialect,
    table_name: String,
    add_if_not_exists: bool,
    add_primary_key: Option<String>,
}

impl SchemaGenerator {
    /// 创建新的 schema 生成器
    pub fn new(dialect: SqlDialect, table_name: String) -> Self {
        Self {
            dialect,
            table_name,
            add_if_not_exists: true,
            add_primary_key: None,
        }
    }

    /// 设置是否添加 IF NOT EXISTS
    pub fn with_if_not_exists(mut self, value: bool) -> Self {
        self.add_if_not_exists = value;
        self
    }

    /// 设置主键列
    pub fn with_primary_key(mut self, column: String) -> Self {
        self.add_primary_key = Some(column);
        self
    }

    /// 生成 CREATE TABLE 语句
    pub fn generate(&self, data: &ExcelData) -> Result<String> {
        let mut sql = String::new();

        // CREATE TABLE 语句开头
        if self.add_if_not_exists {
            match self.dialect {
                SqlDialect::SQLite | SqlDialect::PostgreSQL | SqlDialect::MySQL => {
                    sql.push_str(&format!(
                        "CREATE TABLE IF NOT EXISTS {} (\n",
                        self.dialect.quote_identifier(&self.table_name)
                    ));
                }
                SqlDialect::SqlServer => {
                    sql.push_str(&format!(
                        "IF NOT EXISTS (SELECT * FROM sys.tables WHERE name = '{}')\n",
                        self.table_name
                    ));
                    sql.push_str(&format!(
                        "CREATE TABLE {} (\n",
                        self.dialect.quote_identifier(&self.table_name)
                    ));
                }
                SqlDialect::Oracle => {
                    sql.push_str(&format!(
                        "CREATE TABLE {} (\n",
                        self.dialect.quote_identifier(&self.table_name.to_uppercase())
                    ));
                }
            }
        } else {
            sql.push_str(&format!(
                "CREATE TABLE {} (\n",
                self.dialect.quote_identifier(&self.table_name)
            ));
        }

        // 添加列定义
        let mut column_defs = Vec::new();

        for column in &data.headers {
            let sql_type = TypeInference::infer_column_type(data, column);
            let type_str = sql_type.to_sql_string(&self.dialect);
            let quoted_col = self.dialect.quote_identifier(column);

            let mut def = format!("    {} {}", quoted_col, type_str);

            // 如果是主键列
            if let Some(ref pk_col) = self.add_primary_key {
                if pk_col == column {
                    def.push_str(" PRIMARY KEY");
                }
            }

            column_defs.push(def);
        }

        sql.push_str(&column_defs.join(",\n"));
        sql.push_str("\n);");

        Ok(sql)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ExcelRow;
    use std::collections::HashMap;

    #[test]
    fn test_type_inference() {
        let mut data = ExcelData::new(
            "Sheet1".to_string(),
            vec!["Name".to_string(), "Age".to_string(), "Active".to_string()],
        );

        let mut row1 = HashMap::new();
        row1.insert("Name".to_string(), CellValue::String("Alice".to_string()));
        row1.insert("Age".to_string(), CellValue::Number(30.0));
        row1.insert("Active".to_string(), CellValue::Boolean(true));
        data.add_row(ExcelRow { data: row1 });

        assert_eq!(
            TypeInference::infer_column_type(&data, "Name"),
            SqlType::Varchar(55)
        );
        assert_eq!(
            TypeInference::infer_column_type(&data, "Age"),
            SqlType::Integer
        );
        assert_eq!(
            TypeInference::infer_column_type(&data, "Active"),
            SqlType::Boolean
        );
    }

    #[test]
    fn test_schema_generation() {
        let mut data = ExcelData::new(
            "Sheet1".to_string(),
            vec!["id".to_string(), "name".to_string()],
        );

        let mut row1 = HashMap::new();
        row1.insert("id".to_string(), CellValue::Number(1.0));
        row1.insert("name".to_string(), CellValue::String("Test".to_string()));
        data.add_row(ExcelRow { data: row1 });

        let generator = SchemaGenerator::new(SqlDialect::MySQL, "users".to_string())
            .with_primary_key("id".to_string());

        let sql = generator.generate(&data).unwrap();
        assert!(sql.contains("CREATE TABLE"));
        assert!(sql.contains("`users`"));
        assert!(sql.contains("`id`"));
        assert!(sql.contains("PRIMARY KEY"));
    }
}
