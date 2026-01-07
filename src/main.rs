use clap::{Parser, Subcommand};
use excel_cli::{
    DataFilter, ExcelReader, ExporterConfig, ExporterFactory, FilterCondition, Result,
    SchemaGenerator, SqlDialect,
};
use std::path::Path;

/// Excel 文件转换工具
#[derive(Parser)]
#[command(name = "excel-cli")]
#[command(author = "Your Name <your.email@example.com>")]
#[command(version = "0.3.0")]
#[command(about = "将 Excel 文件转换为 JSON、CSV、SQL 等格式，支持数据过滤和 Schema 生成", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 转换 Excel 文件到指定格式
    Convert {
        /// Excel 文件路径
        #[arg(short, long)]
        input: String,

        /// 输出文件路径
        #[arg(short, long)]
        output: String,

        /// 输出格式 (json, csv, sql, template, html, markdown, xml, yaml)
        #[arg(short, long, default_value = "json")]
        format: String,

        /// 工作表名称（可选，默认使用第一个工作表）
        #[arg(short, long)]
        sheet: Option<String>,

        /// SQL 方言 (mysql, postgresql, sqlite, sqlserver, oracle) - 仅用于 SQL 格式
        #[arg(long, value_name = "DIALECT")]
        sql_dialect: Option<String>,

        /// SQL 表名 - 仅用于 SQL 格式
        #[arg(long, value_name = "TABLE")]
        sql_table: Option<String>,

        /// SQL 语句模式 (insert, update, upsert) - 仅用于 SQL 格式
        #[arg(long, value_name = "MODE", default_value = "insert")]
        sql_mode: String,

        /// 主键列（用逗号分隔，用于 UPDATE 和 UPSERT 模式）
        #[arg(long, value_name = "KEYS")]
        primary_keys: Option<String>,

        /// 要更新的列（用逗号分隔，用于 UPDATE 模式，默认更新所有非主键列）
        #[arg(long, value_name = "COLUMNS")]
        update_columns: Option<String>,

        /// 列名映射，用逗号分隔 (例如: user_id,user_name,user_age) - 仅用于 SQL 格式
        #[arg(long, value_name = "COLUMNS")]
        column_mapping: Option<String>,

        /// 自定义模板文件路径 - 仅用于 template 格式
        #[arg(long, value_name = "PATH")]
        template: Option<String>,

        /// 选择指定的列，用逗号分隔 (例如: Name,Age,City)
        #[arg(long, value_name = "COLUMNS")]
        select: Option<String>,

        /// 排除指定的列，用逗号分隔 (例如: Password,InternalId)
        #[arg(long, value_name = "COLUMNS")]
        exclude: Option<String>,

        /// 过滤条件 (例如: "Age > 30" 或 "City == 北京")
        /// 支持多个条件，每个条件一个参数
        #[arg(long, value_name = "CONDITION")]
        filter: Vec<String>,
    },

    /// 列出 Excel 文件中的所有工作表
    ListSheets {
        /// Excel 文件路径
        #[arg(short, long)]
        input: String,
    },

    /// 生成 CREATE TABLE SQL 语句
    Schema {
        /// Excel 文件路径
        #[arg(short, long)]
        input: String,

        /// 输出文件路径（可选，默认输出到终端）
        #[arg(short, long)]
        output: Option<String>,

        /// 工作表名称（可选，默认使用第一个工作表）
        #[arg(short, long)]
        sheet: Option<String>,

        /// SQL 方言
        #[arg(long, value_name = "DIALECT", default_value = "mysql")]
        sql_dialect: String,

        /// SQL 表名
        #[arg(long, value_name = "TABLE", default_value = "table_name")]
        sql_table: String,

        /// 主键列名（可选）
        #[arg(long, value_name = "COLUMN")]
        primary_key: Option<String>,

        /// 不添加 IF NOT EXISTS
        #[arg(long)]
        no_if_not_exists: bool,
    },

    /// 显示支持的导出格式
    Formats,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Convert {
            input,
            output,
            format,
            sheet,
            sql_dialect,
            sql_table,
            sql_mode,
            primary_keys,
            update_columns,
            column_mapping,
            template,
            select,
            exclude,
            filter,
        } => {
            convert_excel(
                &input,
                &output,
                &format,
                sheet.as_deref(),
                sql_dialect,
                sql_table,
                &sql_mode,
                primary_keys,
                update_columns,
                column_mapping,
                template,
                select,
                exclude,
                filter,
            )?;
        }
        Commands::ListSheets { input } => {
            list_sheets(&input)?;
        }
        Commands::Schema {
            input,
            output,
            sheet,
            sql_dialect,
            sql_table,
            primary_key,
            no_if_not_exists,
        } => {
            generate_schema(
                &input,
                output.as_deref(),
                sheet.as_deref(),
                &sql_dialect,
                &sql_table,
                primary_key,
                !no_if_not_exists,
            )?;
        }
        Commands::Formats => {
            show_formats();
        }
    }

    Ok(())
}

/// 转换 Excel 文件
#[allow(clippy::too_many_arguments)]
fn convert_excel(
    input: &str,
    output: &str,
    format: &str,
    sheet_name: Option<&str>,
    sql_dialect: Option<String>,
    sql_table: Option<String>,
    sql_mode: &str,
    primary_keys: Option<String>,
    update_columns: Option<String>,
    column_mapping: Option<String>,
    template_path: Option<String>,
    select_columns: Option<String>,
    exclude_columns: Option<String>,
    filter_conditions: Vec<String>,
) -> Result<()> {
    // 检查输入文件是否存在
    if !Path::new(input).exists() {
        eprintln!("❌ 错误: 输入文件不存在: {}", input);
        std::process::exit(1);
    }

    println!("📖 正在读取 Excel 文件: {}", input);

    // 创建 Excel 读取器
    let reader = ExcelReader::new(input);

    // 读取工作表数据
    let mut data = reader.read_sheet(sheet_name)?;

    println!(
        "✅ 成功读取工作表 '{}': {} 行 × {} 列",
        data.sheet_name,
        data.row_count(),
        data.column_count()
    );

    // 应用数据过滤
    let has_filter = select_columns.is_some()
        || exclude_columns.is_some()
        || !filter_conditions.is_empty();

    if has_filter {
        println!("🔍 应用数据过滤...");
        let mut filter = DataFilter::new();

        // 选择列
        if let Some(cols) = select_columns {
            let col_list: Vec<String> = cols.split(',').map(|s| s.trim().to_string()).collect();
            println!("   📋 选择列: {}", col_list.join(", "));
            filter = filter.with_select(col_list);
        }

        // 排除列
        if let Some(cols) = exclude_columns {
            let col_list: Vec<String> = cols.split(',').map(|s| s.trim().to_string()).collect();
            println!("   ⛔ 排除列: {}", col_list.join(", "));
            filter = filter.with_exclude(col_list);
        }

        // 过滤条件
        for condition_str in &filter_conditions {
            match FilterCondition::parse(condition_str) {
                Ok(condition) => {
                    println!("   🔎 过滤条件: {}", condition_str);
                    filter = filter.with_condition(condition);
                }
                Err(e) => {
                    eprintln!("❌ 错误: 无效的过滤条件 '{}': {}", condition_str, e);
                    std::process::exit(1);
                }
            }
        }

        // 应用过滤
        data = filter.apply(&data)?;
        println!(
            "✅ 过滤完成: {} 行 × {} 列",
            data.row_count(),
            data.column_count()
        );

        if data.row_count() == 0 {
            println!("⚠️  警告: 过滤后没有数据行");
        }
    }

    // 解析列名映射
    let column_mapping_vec = column_mapping.map(|mapping| {
        mapping
            .split(',')
            .map(|s| s.trim().to_string())
            .collect::<Vec<_>>()
    });

    // 如果是 SQL 格式且提供了列名映射，验证数量
    if format.to_lowercase() == "sql" {
        if let Some(ref mapping) = column_mapping_vec {
            if mapping.len() != data.column_count() {
                eprintln!(
                    "❌ 错误: 列名映射数量({})与 Excel 列数({})不匹配",
                    mapping.len(),
                    data.column_count()
                );
                eprintln!("Excel 列名: {:?}", data.headers);
                eprintln!("映射列名: {:?}", mapping);
                std::process::exit(1);
            }
            println!("📋 列名映射:");
            for (original, mapped) in data.headers.iter().zip(mapping.iter()) {
                println!("   {} -> {}", original, mapped);
            }
        }

        // 显示 SQL 配置
        if let Some(ref dialect) = sql_dialect {
            println!("🗄️  SQL 方言: {}", dialect);
        }
        // 显示 SQL 配置
        if let Some(ref dialect) = sql_dialect {
            println!("🗄️  SQL 方言: {}", dialect);
        }
        if let Some(ref table) = sql_table {
            println!("📊 表名: {}", table);
        }
        println!("📌 SQL 模式: {}", sql_mode);
        if let Some(ref keys) = primary_keys {
            println!("🔑 主键列: {}", keys);
        }
        if let Some(ref cols) = update_columns {
            println!("✏️  更新列: {}", cols);
        }
    }

    // 解析主键和更新列
    let primary_keys_vec = primary_keys.map(|keys| {
        keys.split(',')
            .map(|s| s.trim().to_string())
            .collect::<Vec<_>>()
    });

    let update_columns_vec = update_columns.map(|cols| {
        cols.split(',')
            .map(|s| s.trim().to_string())
            .collect::<Vec<_>>()
    });

    // 创建导出器配置
    let config = ExporterConfig {
        sql_dialect,
        sql_table,
        column_mapping: column_mapping_vec,
        sql_mode: Some(sql_mode.to_string()),
        primary_keys: primary_keys_vec,
        update_columns: update_columns_vec,
        template_path,
    };

    // 创建导出器
    let exporter = ExporterFactory::create(format, config)?;

    println!("📝 正在导出为 {} 格式...", exporter.format_name());

    // 导出数据
    exporter.export(&data, output)?;

    println!("✅ 转换完成! 输出文件: {}", output);

    Ok(())
}

/// 列出所有工作表
fn list_sheets(input: &str) -> Result<()> {
    if !Path::new(input).exists() {
        eprintln!("❌ 错误: 输入文件不存在: {}", input);
        std::process::exit(1);
    }

    let reader = ExcelReader::new(input);
    let sheets = reader.get_sheet_names()?;

    println!("📋 工作表列表:");
    for (idx, sheet) in sheets.iter().enumerate() {
        println!("  {}. {}", idx + 1, sheet);
    }

    Ok(())
}

/// 显示支持的格式
fn show_formats() {
    println!("📦 支持的导出格式:");
    for format in ExporterFactory::supported_formats() {
        println!("  • {}", format);
    }
    println!("\n💡 SQL 格式支持的方言:");
    println!("  • mysql / mariadb");
    println!("  • postgresql / postgres / pg");
    println!("  • sqlite / sqlite3");
    println!("  • sqlserver / mssql / tsql");
    println!("  • oracle");
    println!("\n💡 SQL 模式:");
    println!("  • insert (默认) - 生成 INSERT 语句");
    println!("  • update - 生成 UPDATE 语句");
    println!("  • upsert - 生成 UPSERT/MERGE 语句");
    println!("\n💡 模板格式:");
    println!("  • html / html-table - HTML 表格");
    println!("  • markdown / md / md-table - Markdown 表格");
    println!("  • xml - XML 格式");
    println!("  • yaml / yml - YAML 格式");
    println!("  • template - 自定义 Tera 模板 (需配合 --template 参数)");
    println!("\n💡 提示: 可以通过实现 Exporter trait 添加更多格式支持");
}

/// 生成 CREATE TABLE 语句
fn generate_schema(
    input: &str,
    output: Option<&str>,
    sheet_name: Option<&str>,
    sql_dialect: &str,
    sql_table: &str,
    primary_key: Option<String>,
    add_if_not_exists: bool,
) -> Result<()> {
    // 检查输入文件是否存在
    if !Path::new(input).exists() {
        eprintln!("❌ 错误: 输入文件不存在: {}", input);
        std::process::exit(1);
    }

    println!("📖 正在读取 Excel 文件: {}", input);

    // 创建 Excel 读取器
    let reader = ExcelReader::new(input);

    // 读取工作表数据
    let data = reader.read_sheet(sheet_name)?;

    println!(
        "✅ 成功读取工作表 '{}': {} 行 × {} 列",
        data.sheet_name,
        data.row_count(),
        data.column_count()
    );

    // 解析 SQL 方言
    let dialect = match sql_dialect.to_lowercase().as_str() {
        "mysql" | "mariadb" => SqlDialect::MySQL,
        "postgresql" | "postgres" | "pg" => SqlDialect::PostgreSQL,
        "sqlite" | "sqlite3" => SqlDialect::SQLite,
        "sqlserver" | "mssql" | "tsql" => SqlDialect::SqlServer,
        "oracle" => SqlDialect::Oracle,
        _ => {
            eprintln!("❌ 错误: 不支持的 SQL 方言: {}", sql_dialect);
            eprintln!("支持的方言: mysql, postgresql, sqlite, sqlserver, oracle");
            std::process::exit(1);
        }
    };

    println!("🗄️  SQL 方言: {}", sql_dialect);
    println!("📊 表名: {}", sql_table);

    // 创建 Schema 生成器
    let generator = SchemaGenerator::new(dialect, sql_table.to_string())
        .with_if_not_exists(add_if_not_exists);

    // 设置主键（如果指定）
    let generator = if let Some(pk) = primary_key {
        println!("🔑 主键: {}", pk);
        generator.with_primary_key(pk)
    } else {
        generator
    };

    // 生成 CREATE TABLE 语句
    let sql = generator.generate(&data)?;

    // 输出结果
    match output {
        Some(path) => {
            // 输出到文件
            std::fs::write(path, &sql)?;
            println!("✅ Schema 已生成! 输出文件: {}", path);
        }
        None => {
            // 输出到标准输出
            println!("\n📝 生成的 CREATE TABLE 语句:\n");
            println!("{}", sql);
        }
    }

    Ok(())
}

