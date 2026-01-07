# Excel CLI 架构设计

## 🏗️ 整体架构

```
┌─────────────┐
│   CLI 层    │  main.rs (命令行解析)
└──────┬──────┘
       │
┌──────▼──────────────────────────────┐
│         业务逻辑层                    │
├─────────────┬─────────────┬─────────┤
│  Reader     │  Models     │ Exporter│
│  (读取器)   │  (数据模型)  │ (导出器)│
└─────────────┴─────────────┴─────────┘
       │            │            │
┌──────▼────────────▼────────────▼─────┐
│         外部依赖层                     │
│  calamine  │  serde  │   csv/json   │
└──────────────────────────────────────┘
```

## 📦 模块说明

### 1. CLI 层 (main.rs)

**职责：**
- 解析命令行参数
- 调用业务逻辑
- 处理用户交互和输出

**关键依赖：**
- `clap`: 命令行参数解析

**示例：**
```rust
#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}
```

### 2. 读取器模块 (reader.rs)

**职责：**
- 读取 Excel 文件
- 解析工作表数据
- 将单元格数据转换为统一格式

**API：**
```rust
pub struct ExcelReader {
    pub fn new(file_path: impl AsRef<Path>) -> Self;
    pub fn read_sheet(&self, sheet_name: Option<&str>) -> Result<ExcelData>;
    pub fn get_sheet_names(&self) -> Result<Vec<String>>;
}
```

**关键依赖：**
- `calamine`: Excel 文件解析

### 3. 数据模型层 (models.rs)

**职责：**
- 定义统一的数据结构
- 提供数据访问接口

**核心类型：**

```rust
// 单元格值
pub enum CellValue {
    String(String),
    Number(f64),
    Boolean(bool),
    Empty,
}

// Excel 行
pub struct ExcelRow {
    pub data: HashMap<String, CellValue>,
}

// Excel 表格数据
pub struct ExcelData {
    pub sheet_name: String,
    pub headers: Vec<String>,
    pub rows: Vec<ExcelRow>,
}
```

### 4. 导出器模块 (exporter/)

**职责：**
- 定义导出接口（trait）
- 实现具体导出格式
- 提供导出器工厂

**核心设计：**

```rust
// 导出器 trait
pub trait Exporter {
    fn export(&self, data: &ExcelData, output_path: &str) -> Result<()>;
    fn format_name(&self) -> &'static str;
    fn file_extension(&self) -> &'static str;
}

// 导出器工厂
pub struct ExporterFactory;
impl ExporterFactory {
    pub fn create(format: &str) -> Result<Box<dyn Exporter>>;
    pub fn supported_formats() -> Vec<&'static str>;
}
```

**内置导出器：**
- `JsonExporter`: JSON 格式导出
- `CsvExporter`: CSV 格式导出

### 5. 错误处理层 (error.rs)

**职责：**
- 定义统一的错误类型
- 提供错误转换和包装

**实现：**
```rust
#[derive(Error, Debug)]
pub enum ExcelCliError {
    #[error("无法读取 Excel 文件: {0}")]
    ExcelReadError(String),
    
    #[error("工作表 '{0}' 不存在")]
    SheetNotFound(String),
    
    #[error("导出失败: {0}")]
    ExportError(String),
    
    // ... 更多错误类型
}

pub type Result<T> = std::result::Result<T, ExcelCliError>;
```

## 🔧 扩展新格式

### 方法 1: 实现 Exporter trait

**步骤：**

1. **创建导出器文件**

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
        let mut file = File::create(output_path)?;
        
        writeln!(file, "<?xml version=\"1.0\" encoding=\"UTF-8\"?>")?;
        writeln!(file, "<data>")?;
        
        for row in &data.rows {
            writeln!(file, "  <row>")?;
            for (key, value) in &row.data {
                writeln!(
                    file,
                    "    <{}>{}</{}>",
                    key,
                    value.to_string(),
                    key
                )?;
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

2. **注册导出器**

在 `src/exporter/mod.rs` 中：

```rust
pub mod xml;  // 添加模块声明

impl ExporterFactory {
    pub fn create(format: &str) -> Result<Box<dyn Exporter>> {
        match format.to_lowercase().as_str() {
            "json" => Ok(Box::new(json::JsonExporter::new())),
            "csv" => Ok(Box::new(csv::CsvExporter::new())),
            "xml" => Ok(Box::new(xml::XmlExporter::new())),  // 添加
            _ => Err(ExcelCliError::UnsupportedFormat(format.to_string())),
        }
    }

    pub fn supported_formats() -> Vec<&'static str> {
        vec!["json", "csv", "xml"]  // 添加到列表
    }
}
```

3. **测试新格式**

```bash
cargo build
cargo run -- convert -i data.xlsx -o output.xml -f xml
```

### 方法 2: 扩展 CellValue 类型

如果需要支持更多数据类型，可以扩展 `CellValue` 枚举：

```rust
// 在 models.rs 中
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CellValue {
    String(String),
    Number(f64),
    Boolean(bool),
    Date(String),      // 新增：日期类型
    DateTime(String),  // 新增：日期时间类型
    Empty,
}
```

## 🎯 设计原则

### 1. 单一职责原则 (SRP)

每个模块只负责一个功能：
- Reader 只负责读取
- Exporter 只负责导出
- Models 只负责数据表示

### 2. 开闭原则 (OCP)

**对扩展开放，对修改关闭：**

- 通过实现 `Exporter` trait 添加新格式
- 不需要修改现有代码

### 3. 依赖倒置原则 (DIP)

**依赖抽象而非具体实现：**

```rust
// 依赖 trait 而非具体类型
pub fn export_data(
    data: &ExcelData,
    exporter: &dyn Exporter,  // 依赖抽象
    output: &str
) -> Result<()> {
    exporter.export(data, output)
}
```

### 4. 接口隔离原则 (ISP)

**Exporter trait 只包含必要方法：**

```rust
pub trait Exporter {
    fn export(&self, data: &ExcelData, output_path: &str) -> Result<()>;
    fn format_name(&self) -> &'static str;
    fn file_extension(&self) -> &'static str;
}
```

## 🧪 测试策略

### 单元测试

每个模块都包含单元测试：

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_export() {
        // 创建测试数据
        let data = create_test_data();
        let exporter = JsonExporter::new();
        
        // 执行导出
        let result = exporter.export(&data, "test.json");
        
        // 断言
        assert!(result.is_ok());
        
        // 清理
        std::fs::remove_file("test.json").ok();
    }
}
```

### 集成测试

在 `tests/` 目录下创建集成测试：

```rust
// tests/integration_test.rs
use excel_cli::{ExcelReader, ExporterFactory};

#[test]
fn test_full_conversion_pipeline() {
    let reader = ExcelReader::new("test_data.xlsx");
    let data = reader.read_sheet(None).unwrap();
    
    let exporter = ExporterFactory::create("json").unwrap();
    exporter.export(&data, "output.json").unwrap();
    
    // 验证输出
    assert!(std::path::Path::new("output.json").exists());
}
```

## 📊 性能考虑

### 1. 流式处理

对于大型 Excel 文件，考虑实现流式处理：

```rust
pub trait StreamExporter {
    fn begin(&mut self, headers: &[String]) -> Result<()>;
    fn write_row(&mut self, row: &ExcelRow) -> Result<()>;
    fn finish(&mut self) -> Result<()>;
}
```

### 2. 并行处理

使用 `rayon` 实现并行导出：

```rust
use rayon::prelude::*;

pub fn export_multiple_sheets(
    sheets: Vec<ExcelData>,
    format: &str
) -> Result<()> {
    sheets.par_iter().try_for_each(|sheet| {
        let exporter = ExporterFactory::create(format)?;
        exporter.export(sheet, &format!("{}.json", sheet.sheet_name))
    })
}
```

## 🔐 安全性

### 1. 路径安全

```rust
use std::path::PathBuf;

fn validate_path(path: &str) -> Result<PathBuf> {
    let path = PathBuf::from(path);
    if path.is_absolute() && path.starts_with("..") {
        return Err(ExcelCliError::InvalidPath);
    }
    Ok(path)
}
```

### 2. 资源限制

```rust
const MAX_FILE_SIZE: u64 = 100 * 1024 * 1024; // 100MB

fn check_file_size(path: &Path) -> Result<()> {
    let metadata = std::fs::metadata(path)?;
    if metadata.len() > MAX_FILE_SIZE {
        return Err(ExcelCliError::FileTooLarge);
    }
    Ok(())
}
```

## 🚀 未来扩展方向

1. **更多格式支持**
   - YAML
   - TOML
   - Parquet
   - SQLite

2. **高级功能**
   - 数据过滤
   - 列映射
   - 数据转换
   - 验证规则

3. **性能优化**
   - 流式处理
   - 并行导出
   - 内存优化

4. **用户体验**
   - 进度条
   - 详细日志
   - 交互式选择

## 📚 参考资源

- [Rust Book](https://doc.rust-lang.org/book/)
- [Clap Documentation](https://docs.rs/clap/)
- [Calamine Documentation](https://docs.rs/calamine/)
- [Serde Documentation](https://serde.rs/)

---

有问题或建议？欢迎提交 Issue 或 PR！
