# Excel CLI

一个强大且可扩展的 Rust 命令行工具，用于将 Excel 文件转换为多种格式（JSON、CSV 等）。

## ✨ 特性

- 🚀 **高性能**: 使用 Rust 编写，处理速度快
- 📦 **多格式支持**: 内置支持 JSON、CSV、SQL、HTML、Markdown、XML、YAML 等格式
- 🗄️ **多 SQL 方言**: 支持 MySQL、PostgreSQL、SQLite、SQL Server、Oracle
- 🔄 **SQL 多模式**: 支持 INSERT、UPDATE、UPSERT/MERGE 语句生成
- 📄 **模板引擎**: 使用 Tera 模板引擎支持自定义导出格式
- � **数据过滤**: 支持列选择、列排除和条件过滤
- 📊 **Schema 生成**: 自动生成 CREATE TABLE SQL 语句
- �🔧 **易于扩展**: 通过实现 `Exporter` trait 轻松添加新格式
- 📋 **多工作表**: 支持选择特定工作表或列出所有工作表
- 🏷️ **列名映射**: SQL 导出支持自定义列名映射
- 💪 **类型安全**: 完善的错误处理和类型定义

## 📦 安装

### 从源码构建

```bash
git clone https://github.com/yourusername/excel-cli.git
cd excel-cli
cargo build --release
```

编译完成后，可执行文件位于 `target/release/excel-cli`。

### 直接安装

```bash
cargo install --path .
```

## 🚀 使用方法

### 基本转换

将 Excel 文件转换为 JSON：

```bash
excel-cli convert -i data.xlsx -o output.json -f json
```

将 Excel 文件转换为 CSV：

```bash
excel-cli convert -i data.xlsx -o output.csv -f csv
```

### 指定工作表

默认情况下，工具会读取第一个工作表。你可以指定特定的工作表：

```bash
excel-cli convert -i data.xlsx -o output.json -f json -s "Sheet2"
```

### 导出为 SQL 语句

#### INSERT 语句（默认）

```bash
excel-cli convert -i data.xlsx -o output.sql -f sql --sql-table users
```

#### UPDATE 语句

生成 UPDATE 语句更新现有记录：

```bash
excel-cli convert -i data.xlsx -o update.sql -f sql \
  --sql-table users \
  --sql-mode update \
  --primary-keys ID
```

#### UPSERT/MERGE 语句

生成 UPSERT 语句（存在则更新，不存在则插入）：

```bash
# MySQL 方言
excel-cli convert -i data.xlsx -o upsert.sql -f sql \
  --sql-dialect mysql \
  --sql-table users \
  --sql-mode upsert \
  --primary-keys ID

# PostgreSQL 方言
excel-cli convert -i data.xlsx -o upsert.sql -f sql \
  --sql-dialect postgresql \
  --sql-table users \
  --sql-mode upsert \
  --primary-keys ID
```

#### 指定 SQL 方言

```bash
# PostgreSQL
excel-cli convert -i data.xlsx -o output.sql -f sql --sql-dialect postgresql --sql-table users

# SQLite
excel-cli convert -i data.xlsx -o output.sql -f sql --sql-dialect sqlite --sql-table users

# SQL Server (支持 MERGE 语句)
excel-cli convert -i data.xlsx -o output.sql -f sql --sql-dialect sqlserver --sql-table users

# Oracle (支持 MERGE 语句)
excel-cli convert -i data.xlsx -o output.sql -f sql --sql-dialect oracle --sql-table users
```

#### 使用列名映射

如果 Excel 列名与数据库列名不同，可以使用列名映射：

```bash
# Excel 列名: Name, Age, City
# 映射为: user_name, user_age, user_city
excel-cli convert -i data.xlsx -o output.sql -f sql \
  --sql-table users \
  --sql-dialect mysql \
  --column-mapping "user_name,user_age,user_city"
```

**注意：** 列名映射的数量必须与 Excel 列数完全相同。

📚 **详细文档**: 查看 [UPDATE_UPSERT_GUIDE.md](docs/UPDATE_UPSERT_GUIDE.md) 了解 SQL 语句生成。

### 模板导出

#### HTML 表格

```bash
excel-cli convert -i data.xlsx -o output.html -f html
```

#### Markdown 表格

```bash
excel-cli convert -i data.xlsx -o output.md -f markdown
```

#### XML 格式

```bash
excel-cli convert -i data.xlsx -o output.xml -f xml
```

#### YAML 格式

```bash
excel-cli convert -i data.xlsx -o output.yaml -f yaml
```

#### 自定义模板

创建自己的 Tera 模板文件：

```bash
excel-cli convert -i data.xlsx -o output.txt -f template --template my_template.tera
```

📚 **详细文档**: 查看 [TEMPLATE_GUIDE.md](docs/TEMPLATE_GUIDE.md) 了解模板系统。

### 列出所有工作表

查看 Excel 文件中的所有工作表：

```bash
excel-cli list-sheets -i data.xlsx
```

### 数据过滤

#### 选择特定列

只导出需要的列：

```bash
excel-cli convert -i data.xlsx -o output.json --select "Name,Age,City"
```

#### 排除敏感列

排除不需要的列：

```bash
excel-cli convert -i data.xlsx -o output.json --exclude "Password,InternalId"
```

#### 条件过滤

根据条件筛选数据行：

```bash
# 单个条件
excel-cli convert -i data.xlsx -o output.json --filter "Age > 30"

# 多个条件（AND 关系）
excel-cli convert -i data.xlsx -o output.json \
  --filter "Age > 30" \
  --filter "City == 北京"
```

支持的操作符：`==`, `!=`, `>`, `<`, `>=`, `<=`, `contains`, `not_contains`, `is_empty`, `is_not_empty`

#### 组合使用

```bash
excel-cli convert -i data.xlsx -o filtered.json \
  --select "Name,Age,Salary" \
  --filter "Age >= 30" \
  --filter "Salary > 15000"
```

📚 **详细文档**: 查看 [FILTER_GUIDE.md](FILTER_GUIDE.md) 了解更多过滤功能。

### 生成 CREATE TABLE Schema

自动分析 Excel 数据并生成 CREATE TABLE SQL 语句：

```bash
# 基本用法（输出到终端）
excel-cli schema -i data.xlsx --sql-table users

# 输出到文件
excel-cli schema -i data.xlsx -o schema.sql --sql-table users

# 指定 SQL 方言和主键
excel-cli schema -i data.xlsx -o schema.sql \
  --sql-dialect postgresql \
  --sql-table users \
  --primary-key id
```

支持的 SQL 方言：
- MySQL / MariaDB
- PostgreSQL
- SQLite
- SQL Server
- Oracle

📚 **详细文档**: 查看 [SCHEMA_GUIDE.md](SCHEMA_GUIDE.md) 了解类型推断和 Schema 生成。

### 查看支持的格式

```bash
excel-cli formats
```

输出：
```
📦 支持的导出格式:
  • json
  • csv
  • sql
  • template
  • html
  • markdown
  • xml
  • yaml

💡 SQL 格式支持的方言:
  • mysql / mariadb
  • postgresql / postgres / pg
  • sqlite / sqlite3
  • sqlserver / mssql / tsql
  • oracle

💡 SQL 模式:
  • insert (默认) - 生成 INSERT 语句
  • update - 生成 UPDATE 语句
  • upsert - 生成 UPSERT/MERGE 语句

💡 模板格式:
  • html / html-table - HTML 表格
  • markdown / md / md-table - Markdown 表格
  • xml - XML 格式
  • yaml / yml - YAML 格式
  • template - 自定义 Tera 模板 (需配合 --template 参数)
```

## 📖 命令详解

### `convert` - 转换 Excel 文件

```bash
excel-cli convert [OPTIONS]

选项:
  -i, --input <INPUT>                Excel 文件路径（必需）
  -o, --output <OUTPUT>              输出文件路径（必需）
  -f, --format <FORMAT>              输出格式 [默认: json]
                                     [可选: json, csv, sql, html, markdown, xml, yaml, template]
  -s, --sheet <SHEET>                工作表名称（可选）
  
  SQL 相关选项:
      --sql-dialect <DIALECT>        SQL 方言（仅用于 SQL 格式）
                                     [可选: mysql, postgresql, sqlite, sqlserver, oracle]
      --sql-table <TABLE>            SQL 表名（仅用于 SQL 格式）
      --sql-mode <MODE>              SQL 模式 [默认: insert]
                                     [可选: insert, update, upsert]
      --primary-keys <KEYS>          主键列（用于 UPDATE 和 UPSERT 模式），用逗号分隔
      --update-columns <COLUMNS>     要更新的列（可选，默认更新所有非主键列），用逗号分隔
      --column-mapping <COLUMNS>     列名映射，用逗号分隔（仅用于 SQL 格式）
  
  模板相关选项:
      --template <PATH>              自定义模板文件路径（用于 template 格式）
  
  数据过滤选项:
      --sql-table <TABLE>          SQL 表名（仅用于 SQL 格式）
      --column-mapping <COLUMNS>   列名映射，用逗号分隔（仅用于 SQL 格式）
      --select <COLUMNS>           选择指定的列，用逗号分隔
      --exclude <COLUMNS>          排除指定的列，用逗号分隔
      --filter <CONDITION>         过滤条件，支持多个条件
```

### `schema` - 生成 CREATE TABLE 语句

```bash
excel-cli schema [OPTIONS]

选项:
  -i, --input <INPUT>              Excel 文件路径（必需）
  -o, --output <OUTPUT>            输出文件路径（可选，默认输出到终端）
  -s, --sheet <SHEET>              工作表名称（可选）
      --sql-dialect <DIALECT>      SQL 方言 [默认: mysql]
      --sql-table <TABLE>          SQL 表名 [默认: table_name]
      --primary-key <COLUMN>       主键列名（可选）
      --no-if-not-exists           不添加 IF NOT EXISTS
```

### `list-sheets` - 列出工作表

```bash
excel-cli list-sheets [OPTIONS]

选项:
  -i, --input <INPUT>      Excel 文件路径（必需）
```

### `formats` - 显示支持的格式

```bash
excel-cli formats
```

## 🔧 扩展新格式

项目采用 trait 模式设计，可以轻松添加新的导出格式。

### 1. 创建新的导出器

在 `src/exporter/` 目录下创建新文件，例如 `xml.rs`：

```rust
use crate::error::Result;
use crate::exporter::Exporter;
use crate::models::ExcelData;
use std::fs::File;
use std::io::Write;

pub struct XmlExporter;

impl XmlExporter {
    pub fn new() -> Self {
        Self
    }
}

impl Exporter for XmlExporter {
    fn export(&self, data: &ExcelData, output_path: &str) -> Result<()> {
        // 实现 XML 导出逻辑
        let mut file = File::create(output_path)?;
        
        writeln!(file, "<?xml version=\"1.0\" encoding=\"UTF-8\"?>")?;
        writeln!(file, "<data>")?;
        
        for row in &data.rows {
            writeln!(file, "  <row>")?;
            for (key, value) in &row.data {
                writeln!(file, "    <{}>{}</{}>", key, value.to_string(), key)?;
            }
            writeln!(file, "  </row>")?;
        }
        
        writeln!(file, "</data>")?;
        
        Ok(())
    }

    fn format_name(&self) -> &'static str {
        "XML"
    }

    fn file_extension(&self) -> &'static str {
        "xml"
    }
}
```

### 2. 注册新格式

在 `src/exporter/mod.rs` 中添加：

```rust
pub mod xml;  // 添加模块声明

impl ExporterFactory {
    pub fn create(format: &str) -> Result<Box<dyn Exporter>> {
        match format.to_lowercase().as_str() {
            "json" => Ok(Box::new(json::JsonExporter::new())),
            "csv" => Ok(Box::new(csv::CsvExporter::new())),
            "xml" => Ok(Box::new(xml::XmlExporter::new())),  // 添加新格式
            _ => Err(ExcelCliError::UnsupportedFormat(format.to_string())),
        }
    }

    pub fn supported_formats() -> Vec<&'static str> {
        vec!["json", "csv", "xml"]  // 添加到列表
    }
}
```

完成！现在就可以使用新格式了：

```bash
excel-cli convert -i data.xlsx -o output.xml -f xml
```

## 📁 项目结构

```
excel-cli/
├── Cargo.toml              # 项目配置和依赖
├── README.md               # 项目文档
└── src/
    ├── main.rs             # CLI 入口点
    ├── lib.rs              # 库入口
    ├── error.rs            # 错误定义
    ├── models.rs           # 数据模型
    ├── reader.rs           # Excel 读取器
    └── exporter/           # 导出器模块
        ├── mod.rs          # 导出器 trait 定义
        ├── json.rs         # JSON 导出器
        └── csv.rs          # CSV 导出器
```

## 🛠️ 技术栈

- **[clap](https://github.com/clap-rs/clap)** - 命令行参数解析
- **[calamine](https://github.com/tafia/calamine)** - Excel 文件读取
- **[serde](https://github.com/serde-rs/serde)** - 序列化/反序列化
- **[serde_json](https://github.com/serde-rs/json)** - JSON 处理
- **[csv](https://github.com/BurntSushi/rust-csv)** - CSV 处理
- **[anyhow](https://github.com/dtolnay/anyhow)** - 错误处理
- **[thiserror](https://github.com/dtolnay/thiserror)** - 自定义错误类型

## 📝 示例

### 示例 1: 批量转换

```bash
# 转换多个文件
for file in *.xlsx; do
    excel-cli convert -i "$file" -o "${file%.xlsx}.json" -f json
done
```

### 示例 2: 使用管道

```bash
# 列出工作表并选择性转换
excel-cli list-sheets -i data.xlsx
excel-cli convert -i data.xlsx -o sales.csv -f csv -s "Sales"
```

### 示例 3: 导出为 SQL

```bash
# 导出为 MySQL INSERT 语句
excel-cli convert -i employees.xlsx -o import.sql -f sql \
  --sql-dialect mysql \
  --sql-table employees \
  --column-mapping "emp_id,emp_name,emp_age,emp_dept"

# 导出为 PostgreSQL INSERT 语句
excel-cli convert -i products.xlsx -o products.sql -f sql \
  --sql-dialect postgresql \
  --sql-table products
```

## 🤝 贡献

欢迎贡献！请遵循以下步骤：

1. Fork 本仓库
2. 创建特性分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 开启 Pull Request

## 📄 许可证

本项目采用 MIT 许可证 - 查看 [LICENSE](LICENSE) 文件了解详情。

## 🐛 问题反馈

如果遇到问题或有功能建议，请[提交 Issue](https://github.com/yourusername/excel-cli/issues)。

## 📚 文档

- **[README.md](README.md)** - 项目介绍和基本使用（本文件）
- **[QUICKSTART.md](QUICKSTART.md)** - 快速入门指南
- **[EXAMPLES.md](EXAMPLES.md)** - 详细使用示例
- **[FILTER_GUIDE.md](FILTER_GUIDE.md)** - 数据过滤和列选择指南 ⭐ 新功能
- **[SCHEMA_GUIDE.md](SCHEMA_GUIDE.md)** - CREATE TABLE 生成指南 ⭐ 新功能
- **[SQL_EXPORT_GUIDE.md](SQL_EXPORT_GUIDE.md)** - SQL 导出功能详解
- **[SQL_DEMO.md](SQL_DEMO.md)** - SQL 导出快速演示
- **[ARCHITECTURE.md](ARCHITECTURE.md)** - 架构设计文档
- **[PROJECT_SUMMARY.md](PROJECT_SUMMARY.md)** - 项目总结
- **[CHANGELOG.md](CHANGELOG.md)** - 更新日志

## 📚 更多资源

- [Rust 官方文档](https://doc.rust-lang.org/)
- [Calamine 文档](https://docs.rs/calamine/)
- [Clap 文档](https://docs.rs/clap/)

## 📖 相关文档

- **[SQL_EXPORT_GUIDE.md](SQL_EXPORT_GUIDE.md)** - SQL 导出功能完整指南
- **[EXAMPLES.md](EXAMPLES.md)** - 更多使用示例
- **[ARCHITECTURE.md](ARCHITECTURE.md)** - 了解项目架构

---

⭐ 如果这个项目对你有帮助，请给个 Star！
