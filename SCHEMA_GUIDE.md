# CREATE TABLE Schema 生成指南

本指南介绍如何使用 excel-cli 的 schema 命令自动生成 CREATE TABLE SQL 语句。

## 目录

- [功能概述](#功能概述)
- [基本用法](#基本用法)
- [类型推断](#类型推断)
- [SQL 方言支持](#sql-方言支持)
- [高级选项](#高级选项)
- [实际应用场景](#实际应用场景)
- [与 SQL INSERT 导出配合使用](#与-sql-insert-导出配合使用)

## 功能概述

schema 命令可以：

- 📊 **分析 Excel 数据** - 自动检测列的数据类型
- 🗄️ **生成 CREATE TABLE 语句** - 支持 5 种 SQL 方言
- 🔑 **定义主键** - 指定主键列
- 🎯 **类型映射** - 智能推断最合适的 SQL 类型
- 💾 **输出灵活** - 可输出到文件或终端

## 基本用法

### 最简单的用法

```bash
excel-cli schema -i users.xlsx
```

这将：
- 使用 MySQL 方言（默认）
- 表名为 `table_name`（默认）
- 输出到终端
- 包含 `IF NOT EXISTS`

### 指定输出文件

```bash
excel-cli schema -i users.xlsx -o schema.sql
```

### 完整示例

```bash
excel-cli schema \
  -i users.xlsx \
  -o create_users.sql \
  --sql-dialect postgresql \
  --sql-table users \
  --primary-key id
```

## 类型推断

excel-cli 会智能分析 Excel 数据并推断最合适的 SQL 类型。

### 推断规则

| Excel 数据类型 | 示例值 | 推断的 SQL 类型 |
|---------------|--------|----------------|
| 整数 | 1, 42, -100 | `INT` / `INTEGER` |
| 大整数 | 2147483648 | `BIGINT` |
| 小数 | 3.14, -0.5 | `FLOAT` / `DOUBLE` |
| 布尔值 | TRUE, FALSE, true, false | `BOOLEAN` / `BIT` |
| 日期 | 2023-01-15 | `DATE` |
| 日期时间 | 2023-01-15 14:30:00 | `DATETIME` / `TIMESTAMP` |
| 短字符串 | "Hello", "张三" | `VARCHAR(n)` |
| 长字符串 | 长文本内容 | `TEXT` |
| 空值 | (空) | `VARCHAR(255)` (默认) |

### 类型推断示例

假设有如下 Excel 数据：

| ID | Name | Age | Salary | Active | CreatedAt |
|----|------|-----|--------|--------|-----------|
| 1 | 张三 | 28 | 15000.50 | TRUE | 2023-01-15 |
| 2 | 李四 | 35 | 22000.00 | TRUE | 2023-02-20 |

生成的 Schema：

```sql
CREATE TABLE IF NOT EXISTS `users` (
    `ID` INT,
    `Name` VARCHAR(56),      -- 根据最长值确定长度
    `Age` INT,
    `Salary` DOUBLE,          -- 包含小数，使用 DOUBLE
    `Active` BOOLEAN,
    `CreatedAt` DATE
);
```

### 字符串长度计算

- VARCHAR 长度 = 最长字符串长度 × 2（考虑 UTF-8 编码）
- 如果计算长度超过 255，使用 TEXT 类型
- 空列默认为 VARCHAR(255)

## SQL 方言支持

excel-cli 支持 5 种主流 SQL 方言，每种方言的类型映射略有不同。

### MySQL

```bash
excel-cli schema -i data.xlsx --sql-dialect mysql --sql-table users
```

输出：

```sql
CREATE TABLE IF NOT EXISTS `users` (
    `id` INT,
    `name` VARCHAR(100),
    `age` INT,
    `salary` DOUBLE,
    `is_active` BOOLEAN,
    `created_at` DATETIME
);
```

特点：
- 使用反引号 `` ` `` 包裹标识符
- BOOLEAN 类型
- DATETIME 类型

### PostgreSQL

```bash
excel-cli schema -i data.xlsx --sql-dialect postgresql --sql-table users
```

输出：

```sql
CREATE TABLE IF NOT EXISTS "users" (
    "id" INTEGER,
    "name" VARCHAR(100),
    "age" INTEGER,
    "salary" DOUBLE PRECISION,
    "is_active" BOOLEAN,
    "created_at" TIMESTAMP
);
```

特点：
- 使用双引号 `"` 包裹标识符
- INTEGER 代替 INT
- DOUBLE PRECISION 代替 DOUBLE
- TIMESTAMP 代替 DATETIME

### SQLite

```bash
excel-cli schema -i data.xlsx --sql-dialect sqlite --sql-table users
```

输出：

```sql
CREATE TABLE IF NOT EXISTS "users" (
    "id" INTEGER,
    "name" TEXT,                -- VARCHAR 转为 TEXT
    "age" INTEGER,
    "salary" REAL,              -- DOUBLE 转为 REAL
    "is_active" INTEGER,        -- BOOLEAN 转为 INTEGER
    "created_at" TEXT           -- DATE/DATETIME 转为 TEXT
);
```

特点：
- 类型系统简化
- VARCHAR → TEXT
- DOUBLE → REAL
- BOOLEAN → INTEGER
- DATE/DATETIME → TEXT

### SQL Server

```bash
excel-cli schema -i data.xlsx --sql-dialect sqlserver --sql-table users
```

输出：

```sql
CREATE TABLE IF NOT EXISTS [users] (
    [id] INT,
    [name] NVARCHAR(100),       -- 使用 NVARCHAR 支持 Unicode
    [age] INT,
    [salary] FLOAT,
    [is_active] BIT,            -- BOOLEAN 转为 BIT
    [created_at] DATETIME
);
```

特点：
- 使用方括号 `[]` 包裹标识符
- NVARCHAR 支持 Unicode
- BIT 代替 BOOLEAN
- FLOAT 代替 DOUBLE

### Oracle

```bash
excel-cli schema -i data.xlsx --sql-dialect oracle --sql-table users
```

输出：

```sql
CREATE TABLE IF NOT EXISTS "USERS" (
    "ID" NUMBER,                -- INT 转为 NUMBER
    "NAME" VARCHAR2(100),       -- VARCHAR 转为 VARCHAR2
    "AGE" NUMBER,
    "SALARY" FLOAT,
    "IS_ACTIVE" NUMBER(1),      -- BOOLEAN 转为 NUMBER(1)
    "CREATED_AT" DATE
);
```

特点：
- 标识符自动转为大写
- NUMBER 类型系统
- VARCHAR2 代替 VARCHAR
- NUMBER(1) 代替 BOOLEAN

## 高级选项

### 指定主键

```bash
excel-cli schema -i users.xlsx --sql-table users --primary-key id
```

输出：

```sql
CREATE TABLE IF NOT EXISTS `users` (
    `id` INT PRIMARY KEY,       -- 添加 PRIMARY KEY
    `name` VARCHAR(100),
    `age` INT
);
```

### 移除 IF NOT EXISTS

默认情况下会添加 `IF NOT EXISTS`，如果不需要：

```bash
excel-cli schema -i users.xlsx --sql-table users --no-if-not-exists
```

输出：

```sql
CREATE TABLE `users` (          -- 没有 IF NOT EXISTS
    `id` INT,
    `name` VARCHAR(100),
    `age` INT
);
```

### 指定工作表

```bash
excel-cli schema -i data.xlsx --sheet "用户信息" --sql-table users
```

### 组合使用

```bash
excel-cli schema \
  -i company_data.xlsx \
  -o schema/employees.sql \
  --sheet "员工表" \
  --sql-dialect postgresql \
  --sql-table employees \
  --primary-key employee_id \
  --no-if-not-exists
```

## 实际应用场景

### 场景 1：快速建表

从 Excel 设计稿快速生成数据库表结构：

```bash
# 1. 生成 Schema
excel-cli schema -i design.xlsx -o schema.sql --sql-table users --primary-key id

# 2. 在数据库中执行
mysql -u root -p mydb < schema.sql
```

### 场景 2：多表导入

为多个工作表生成独立的 Schema：

```bash
# 用户表
excel-cli schema -i data.xlsx --sheet "用户" -o users_schema.sql --sql-table users --primary-key user_id

# 订单表
excel-cli schema -i data.xlsx --sheet "订单" -o orders_schema.sql --sql-table orders --primary-key order_id

# 产品表
excel-cli schema -i data.xlsx --sheet "产品" -o products_schema.sql --sql-table products --primary-key product_id
```

### 场景 3：数据库迁移

从 MySQL 迁移到 PostgreSQL：

```bash
# 1. 从现有 MySQL 数据导出到 Excel
# 2. 生成 PostgreSQL Schema
excel-cli schema -i mysql_export.xlsx -o pg_schema.sql --sql-dialect postgresql --sql-table mytable

# 3. 在 PostgreSQL 中创建表
psql -U postgres -d mydb -f pg_schema.sql
```

### 场景 4：开发环境初始化

为新加入的开发人员快速搭建本地数据库：

```bash
#!/bin/bash
# setup_db.sh

# 生成所有表的 Schema
excel-cli schema -i sample_data.xlsx --sheet "用户" -o schema/users.sql --sql-table users --primary-key id
excel-cli schema -i sample_data.xlsx --sheet "订单" -o schema/orders.sql --sql-table orders --primary-key id

# 创建数据库和表
mysql -u root -p << EOF
CREATE DATABASE IF NOT EXISTS dev_db;
USE dev_db;
SOURCE schema/users.sql;
SOURCE schema/orders.sql;
EOF

echo "✅ 数据库初始化完成"
```

## 与 SQL INSERT 导出配合使用

Schema 生成和 SQL INSERT 导出可以完美配合，实现完整的数据迁移流程。

### 完整工作流

#### 步骤 1：生成 CREATE TABLE

```bash
excel-cli schema \
  -i users.xlsx \
  -o 01_create_table.sql \
  --sql-dialect mysql \
  --sql-table users \
  --primary-key id
```

#### 步骤 2：生成 INSERT 语句

```bash
excel-cli convert \
  -i users.xlsx \
  -o 02_insert_data.sql \
  -f sql \
  --sql-dialect mysql \
  --sql-table users
```

#### 步骤 3：执行 SQL

```bash
# 合并 SQL 文件
cat 01_create_table.sql 02_insert_data.sql > full_migration.sql

# 执行
mysql -u root -p mydb < full_migration.sql
```

### 自动化脚本示例

创建一个脚本 `import_excel_to_db.sh`：

```bash
#!/bin/bash

EXCEL_FILE=$1
TABLE_NAME=$2
DIALECT=${3:-mysql}
PRIMARY_KEY=${4:-id}

if [ -z "$EXCEL_FILE" ] || [ -z "$TABLE_NAME" ]; then
    echo "用法: $0 <Excel文件> <表名> [方言] [主键]"
    exit 1
fi

echo "📊 处理 Excel 文件: $EXCEL_FILE"
echo "🗄️  目标表: $TABLE_NAME"
echo "💾 SQL 方言: $DIALECT"

# 1. 生成 Schema
echo "1️⃣  生成 CREATE TABLE..."
excel-cli schema \
  -i "$EXCEL_FILE" \
  -o "${TABLE_NAME}_schema.sql" \
  --sql-dialect "$DIALECT" \
  --sql-table "$TABLE_NAME" \
  --primary-key "$PRIMARY_KEY"

# 2. 生成 INSERT 语句
echo "2️⃣  生成 INSERT 语句..."
excel-cli convert \
  -i "$EXCEL_FILE" \
  -o "${TABLE_NAME}_data.sql" \
  -f sql \
  --sql-dialect "$DIALECT" \
  --sql-table "$TABLE_NAME"

# 3. 合并文件
echo "3️⃣  合并 SQL 文件..."
cat "${TABLE_NAME}_schema.sql" "${TABLE_NAME}_data.sql" > "${TABLE_NAME}_full.sql"

echo "✅ 完成！生成的文件:"
echo "   - ${TABLE_NAME}_schema.sql (CREATE TABLE)"
echo "   - ${TABLE_NAME}_data.sql (INSERT)"
echo "   - ${TABLE_NAME}_full.sql (完整脚本)"
```

使用：

```bash
chmod +x import_excel_to_db.sh
./import_excel_to_db.sh users.xlsx users mysql id
```

### 带过滤的数据迁移

```bash
# 1. 生成完整表结构（基于所有列）
excel-cli schema \
  -i users.xlsx \
  -o users_schema.sql \
  --sql-table users \
  --primary-key id

# 2. 只导入活跃用户的部分字段
excel-cli convert \
  -i users.xlsx \
  -o active_users_data.sql \
  -f sql \
  --sql-table users \
  --select "id,name,email,created_at" \
  --filter "is_active == true" \
  --filter "created_at >= 2023-01-01"
```

## 最佳实践

### 1. 数据质量检查

在生成 Schema 前，确保 Excel 数据：
- 第一行是列名
- 列名不包含特殊字符
- 数据类型一致（同一列的所有值类型相同）
- 没有合并单元格

### 2. 类型验证

生成 Schema 后，检查：
- VARCHAR 长度是否合理
- 数值类型是否正确（INT vs BIGINT vs FLOAT）
- 日期类型是否符合需求

### 3. 主键选择

- 始终指定主键（如果有唯一标识列）
- 主键列应该是非空的
- 主键列名应该在 Excel 中存在

### 4. 多环境支持

为不同数据库生成不同的 Schema：

```bash
# 开发环境 - SQLite
excel-cli schema -i data.xlsx -o dev_schema.sql --sql-dialect sqlite --sql-table users

# 生产环境 - PostgreSQL
excel-cli schema -i data.xlsx -o prod_schema.sql --sql-dialect postgresql --sql-table users
```

### 5. 版本控制

将生成的 Schema 文件纳入版本控制：

```bash
mkdir -p migrations/schemas
excel-cli schema -i v1.0_design.xlsx -o migrations/schemas/001_create_users.sql
git add migrations/schemas/001_create_users.sql
git commit -m "Add users table schema v1.0"
```

## 常见问题

### Q: 如何处理中文列名？

A: excel-cli 完全支持中文列名，但在生产环境建议使用英文列名：

```sql
-- 自动生成（中文列名）
CREATE TABLE `users` (
    `用户ID` INT,
    `用户名` VARCHAR(100),
    `年龄` INT
);

-- 建议手动修改为
CREATE TABLE `users` (
    `user_id` INT,
    `username` VARCHAR(100),
    `age` INT
);
```

### Q: 类型推断不准确怎么办？

A: 生成 Schema 后手动修改。例如：

```sql
-- 自动生成
`phone` VARCHAR(40)

-- 修改为更合适的类型
`phone` VARCHAR(20)
```

### Q: 如何添加 NOT NULL 约束？

A: 当前版本不支持自动添加 NOT NULL，需要手动添加：

```sql
CREATE TABLE `users` (
    `id` INT PRIMARY KEY,
    `name` VARCHAR(100) NOT NULL,
    `email` VARCHAR(255) NOT NULL UNIQUE,
    `age` INT
);
```

### Q: 支持外键吗？

A: 当前版本不支持外键，需要手动添加：

```sql
-- 生成后添加外键
ALTER TABLE orders
ADD CONSTRAINT fk_user
FOREIGN KEY (user_id) REFERENCES users(id);
```

### Q: 如何处理索引？

A: 生成 Schema 后手动添加索引：

```sql
-- 在 CREATE TABLE 后添加
CREATE INDEX idx_email ON users(email);
CREATE INDEX idx_created_at ON users(created_at);
```

## 小结

CREATE TABLE Schema 生成功能让您可以：

- ⚡ **快速建表** - 无需手写 DDL
- 🎯 **类型准确** - 智能推断最合适的类型
- 🌍 **跨数据库** - 支持 5 种主流数据库
- 🔄 **配合使用** - 与 INSERT 导出完美结合
- 🚀 **自动化** - 适合脚本和 CI/CD 流程

结合数据过滤功能，您可以实现从 Excel 到数据库的完整自动化工作流！
