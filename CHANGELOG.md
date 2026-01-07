# 更新日志 (Changelog)

本项目遵循 [语义化版本](https://semver.org/lang/zh-CN/)。

## [0.4.0] - 2026-01-09

### ✨ 新增功能

#### SQL UPDATE 和 UPSERT 语句支持
- **新增 SQL 模式** (`--sql-mode`)
  - `insert` (默认) - 生成 INSERT 语句
  - `update` - 生成 UPDATE 语句
  - `upsert` - 生成 UPSERT/MERGE 语句

- **UPDATE 模式**
  - 根据主键列生成 UPDATE 语句
  - 支持多个主键列（复合主键）
  - 可指定要更新的列 (`--update-columns`)
  - 默认更新所有非主键列
  - 示例：`UPDATE users SET name='张三' WHERE id=1;`

- **UPSERT 模式**
  - 生成方言特定的 UPSERT/MERGE 语句
  - MySQL: `ON DUPLICATE KEY UPDATE`
  - PostgreSQL/SQLite: `ON CONFLICT DO UPDATE`
  - SQL Server/Oracle: `MERGE INTO` 语句
  - 自动处理主键冲突

- **新增命令行参数**
  - `--sql-mode <MODE>`: 指定 SQL 生成模式
  - `--primary-keys <KEYS>`: 指定主键列（逗号分隔）
  - `--update-columns <COLUMNS>`: 指定要更新的列（可选）

#### 模板导出支持
- **Tera 模板引擎集成**
  - 使用 Tera 1.19 (类似 Jinja2 语法)
  - 支持自定义模板
  - 提供丰富的模板变量和过滤器

- **4 种内置模板格式**
  - `html` / `html-table` - 完整的 HTML 表格页面（带 CSS 样式）
  - `markdown` / `md` - Markdown 表格格式
  - `xml` - 标准 XML 结构
  - `yaml` / `yml` - YAML 格式

- **自定义模板**
  - 使用 `--template` 参数指定自定义模板文件
  - 可访问的模板变量：
    - `sheet_name`: 工作表名称
    - `row_count`: 行数
    - `column_count`: 列数
    - `headers`: 列标题列表
    - `rows`: 数据行列表
  - 支持 Tera 所有功能（循环、条件、过滤器等）

- **模板示例**
  - HTML 表格：带悬停效果和响应式设计
  - Markdown：GitHub 兼容的表格格式
  - XML：结构化数据交换格式
  - YAML：配置文件友好格式

#### 新增文档
- `docs/UPDATE_UPSERT_GUIDE.md` - UPDATE 和 UPSERT 详细使用指南
- `docs/TEMPLATE_GUIDE.md` - 模板导出完整指南

### 🔧 改进

- 扩展 `ExporterConfig` 以支持新的 SQL 参数
- 更新 `ExporterFactory` 以处理模板格式
- 改进 `formats` 命令输出，显示所有支持的格式和模式
- 增强错误消息（如缺少主键时的提示）

### 📦 依赖更新

- 新增 `tera = "1.19"` - 模板引擎

### 🐛 修复

- 修复 SQL 导出时的列名引用问题
- 改进类型推断的准确性

---

## [0.3.0] - 2026-01-08

### ✨ 新增功能

#### 数据过滤功能
- **列选择** (`--select`)
  - 选择需要导出的特定列
  - 支持多列用逗号分隔
  - 列顺序影响输出顺序
  
- **列排除** (`--exclude`)
  - 排除不需要的列（如敏感信息）
  - 支持多列用逗号分隔
  - 适合移除少量列的场景

- **行过滤** (`--filter`)
  - 根据条件筛选数据行
  - 支持多个过滤条件（AND 关系）
  - 支持 10 种操作符：
    - 比较：`==`, `!=`, `>`, `<`, `>=`, `<=`
    - 字符串：`contains`, `not_contains`
    - 空值：`is_empty`, `is_not_empty`

- **组合过滤**
  - 可同时使用列选择、列排除和行过滤
  - 过滤顺序：行过滤 → 列选择/排除
  - 支持所有导出格式（JSON、CSV、SQL）

#### CREATE TABLE Schema 生成
- **新增 `schema` 子命令**
  - 自动分析 Excel 数据生成 CREATE TABLE 语句
  - 智能类型推断（INT、VARCHAR、FLOAT、BOOLEAN、DATE 等）
  - 支持 5 种 SQL 方言（MySQL、PostgreSQL、SQLite、SQL Server、Oracle）
  
- **类型推断系统**
  - 整数检测（INT / BIGINT）
  - 浮点数检测（FLOAT / DOUBLE）
  - 布尔值检测（BOOLEAN / BIT）
  - 日期和日期时间检测（DATE / DATETIME / TIMESTAMP）
  - 字符串长度分析（VARCHAR(n) / TEXT）
  - 空值处理（默认 VARCHAR(255)）

- **Schema 生成选项**
  - `--primary-key`: 指定主键列
  - `--sql-table`: 指定表名
  - `--sql-dialect`: 指定 SQL 方言
  - `--no-if-not-exists`: 移除 IF NOT EXISTS 子句
  - `-o / --output`: 输出到文件或终端

#### 模块架构
- 新增 `src/filter.rs` 模块
  - DataFilter 结构体
  - FilterCondition 条件解析
  - FilterOperator 操作符枚举
  - 完整的单元测试（5 个测试）

- 新增 `src/schema.rs` 模块
  - TypeInference 类型推断
  - SqlType 类型枚举
  - SchemaGenerator 生成器
  - 方言特定类型映射
  - 完整的单元测试（2 个测试）

### 🔧 改进

- **CLI 增强**
  - 增强的进度提示和日志输出
  - 过滤操作的详细信息显示
  - 更友好的错误消息
  
- **代码质量**
  - 所有测试通过（12 个单元测试）
  - 改进的错误处理
  - 更好的类型安全
  - 代码文档完善

- **API 改进**
  - 公开 `quote_identifier` 方法供 schema 模块使用
  - 改进的模块导出结构
  - 更清晰的依赖关系

### 📝 文档

- **FILTER_GUIDE.md** - 数据过滤完整指南
  - 列选择和排除详解
  - 行过滤操作符说明
  - 组合使用示例
  - 实际应用场景（10+ 个示例）
  - 性能建议和常见问题

- **SCHEMA_GUIDE.md** - CREATE TABLE 生成指南
  - 类型推断规则详解
  - 5 种 SQL 方言对比
  - 高级选项说明
  - 与 SQL INSERT 配合使用
  - 完整工作流示例
  - 自动化脚本模板

- 更新 **README.md**
  - 添加数据过滤功能说明
  - 添加 Schema 生成功能说明
  - 更新特性列表
  - 更新命令详解
  - 新增文档链接

### 🧪 测试

- 新增 5 个过滤功能测试
  - 条件解析测试
  - 过滤条件应用测试
  - 列选择测试
  
- 新增 2 个 Schema 生成测试
  - 类型推断测试
  - Schema 生成测试

- 所有 12 个单元测试全部通过

### 🎯 使用示例

**数据过滤:**
```bash
# 选择列 + 过滤行
excel-cli convert -i data.xlsx -o output.json \
  --select "Name,Age,Salary" \
  --filter "Age >= 30" \
  --filter "Salary > 20000"
```

**Schema 生成:**
```bash
# 生成 CREATE TABLE
excel-cli schema -i data.xlsx -o schema.sql \
  --sql-dialect postgresql \
  --sql-table users \
  --primary-key id
```

### 📊 统计

- 新增代码行数：~650 行
- 新增模块：2 个（filter, schema）
- 新增测试：7 个
- 新增文档：2 个完整指南
- 总测试数：12 个（全部通过）

---

## [0.2.0] - 2026-01-07

### ✨ 新增功能

#### SQL 导出支持
- **SQL INSERT 语句导出**
  - 支持将 Excel 数据导出为 SQL INSERT 语句
  - 支持批量 INSERT（MySQL、PostgreSQL、SQLite）
  - 支持单条 INSERT（SQL Server、Oracle）

- **多 SQL 方言支持**
  - MySQL / MariaDB（使用 \`backticks\` 引用标识符）
  - PostgreSQL（使用 "double quotes" 引用标识符）
  - SQLite（使用 "double quotes" 引用标识符）
  - SQL Server（使用 [brackets] 引用标识符）
  - Oracle（使用 "UPPERCASE" 引用标识符）

- **列名映射功能**
  - 支持自定义列名映射（通过 `--column-mapping` 参数）
  - 使用逗号分隔列名
  - 自动验证列名数量与 Excel 列数匹配
  - 显示列名映射关系

#### CLI 增强
- 新增 `--sql-dialect` 参数：指定 SQL 方言
- 新增 `--sql-table` 参数：指定目标表名
- 新增 `--column-mapping` 参数：指定列名映射
- 改进 `formats` 命令：显示支持的 SQL 方言列表

#### 数据类型处理
- 字符串：自动添加单引号并转义内部单引号
- 数字：智能识别整数和浮点数
- 布尔值：根据方言转换为适当格式（TRUE/FALSE 或 1/0）
- 空值：统一转换为 NULL
- 日期时间：转换为字符串格式

### 📝 文档

- **SQL_EXPORT_GUIDE.md** - SQL 导出功能详细指南
  - 支持的 SQL 方言说明
  - 列名映射使用方法
  - 完整示例和最佳实践
  - 常见错误和解决方法

- 更新 **README.md** 包含 SQL 导出示例
- 更新命令行帮助文档

### 🧪 测试

- 新增 SQL 导出器单元测试
  - 方言标识符引用测试
  - 列名映射验证测试
  - 数据类型格式化测试
  - SQL 导出功能测试

### 📊 统计信息

- 总代码量：~1000 行（+350 行）
- 测试覆盖：7 个单元测试（+4 个）
- 文档页数：6 个主要文档（+1 个）
- 支持格式：3 种（JSON、CSV、SQL）
- SQL 方言：5 种

---

## [0.1.0] - 2026-01-07

### ✨ 新增功能

#### 核心功能
- **Excel 文件读取**
  - 支持 .xlsx 格式文件读取
  - 支持多工作表
  - 自动解析表头和数据行
  - 支持多种单元格类型（字符串、数字、布尔、日期等）

- **导出格式**
  - JSON 格式导出（支持格式化输出）
  - CSV 格式导出（支持自定义分隔符）

- **CLI 命令**
  - `convert` - 转换 Excel 文件到指定格式
  - `list-sheets` - 列出 Excel 文件中的所有工作表
  - `formats` - 显示支持的导出格式

#### 架构设计
- 基于 Trait 的可扩展导出器架构
- 清晰的模块划分（reader、exporter、models、error）
- 统一的错误处理机制
- 类型安全的数据模型

#### 开发工具
- 完整的单元测试框架
- 代码注释和文档
- 示例和使用指南

### 📝 文档

- **README.md** - 完整的项目介绍和使用说明
- **QUICKSTART.md** - 快速入门指南
- **EXAMPLES.md** - 详细的使用示例
- **ARCHITECTURE.md** - 架构设计文档
- **PROJECT_SUMMARY.md** - 项目总结
- **LICENSE** - MIT 许可证

### 🛠️ 技术栈

- Rust 2021 Edition
- clap 4.5 - CLI 框架
- calamine 0.25 - Excel 读取
- serde 1.0 - 序列化
- serde_json 1.0 - JSON 处理
- csv 1.3 - CSV 处理
- anyhow 1.0 - 错误处理
- thiserror 1.0 - 自定义错误

### 📊 统计信息

- 总代码量：~650 行
- 测试覆盖：3 个单元测试
- 文档页数：5 个主要文档
- 支持格式：2 种（JSON、CSV）

---

## [未来计划]

### 版本 0.2.0（计划中）
- [ ] 支持更多导出格式（XML、YAML、TOML）
- [ ] 数据过滤功能
- [ ] 列映射和重命名
- [ ] 进度条显示
- [ ] 流式处理大文件

### 版本 0.3.0（计划中）
- [ ] 并行处理多个文件
- [ ] 数据验证规则
- [ ] 自定义转换函数
- [ ] 配置文件支持
- [ ] 增量更新模式

### 版本 1.0.0（远期）
- [ ] 生产级稳定性
- [ ] 完整的集成测试
- [ ] 性能基准测试
- [ ] 插件系统
- [ ] Web UI

---

## 维护说明

### 版本号规则

- **主版本号（Major）**: 不兼容的 API 修改
- **次版本号（Minor）**: 向下兼容的功能新增
- **修订号（Patch）**: 向下兼容的问题修复

### 发布流程

1. 更新版本号（Cargo.toml）
2. 更新 CHANGELOG.md
3. 运行所有测试：`cargo test`
4. 构建 Release：`cargo build --release`
5. 创建 Git 标签：`git tag -a v0.1.0 -m "Release v0.1.0"`
6. 推送标签：`git push origin v0.1.0`
7. 发布到 crates.io：`cargo publish`

### 贡献者

感谢所有为本项目做出贡献的开发者！

---

**注意：** 本项目目前处于初始开发阶段，API 可能会发生变化。
