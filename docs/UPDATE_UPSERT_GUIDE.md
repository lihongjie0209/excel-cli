# UPDATE 和 UPSERT 语句使用指南

excel-cli 支持三种 SQL 生成模式：`INSERT`（默认）、`UPDATE` 和 `UPSERT`。

## 1. INSERT 模式（默认）

生成标准的 INSERT 语句。

### 使用方法

```bash
excel-cli convert -i users.xlsx -o users.sql -f sql --sql-dialect mysql --sql-table users
```

或显式指定：

```bash
excel-cli convert -i users.xlsx -o users.sql -f sql \
  --sql-dialect mysql \
  --sql-table users \
  --sql-mode insert
```

### 生成示例

```sql
INSERT INTO `users` (`ID`, `Name`, `Email`) VALUES (1, '张三', 'zhangsan@example.com');
INSERT INTO `users` (`ID`, `Name`, `Email`) VALUES (2, '李四', 'lisi@example.com');
```

---

## 2. UPDATE 模式

生成 UPDATE 语句，根据主键更新记录。

### 使用方法

**必须指定主键列** （可以是单个或多个列）：

```bash
excel-cli convert -i users.xlsx -o users_update.sql -f sql \
  --sql-dialect mysql \
  --sql-table users \
  --sql-mode update \
  --primary-keys ID
```

### 多个主键列

使用逗号分隔：

```bash
excel-cli convert -i users.xlsx -o users_update.sql -f sql \
  --sql-dialect mysql \
  --sql-table users \
  --sql-mode update \
  --primary-keys ID,Email
```

### 指定更新列

默认更新所有非主键列。可以使用 `--update-columns` 指定要更新的列：

```bash
excel-cli convert -i users.xlsx -o users_update.sql -f sql \
  --sql-dialect mysql \
  --sql-table users \
  --sql-mode update \
  --primary-keys ID \
  --update-columns Name,Email
```

### 生成示例

```sql
UPDATE `users` SET `Name` = '张三', `Email` = 'zhangsan@example.com' WHERE `ID` = 1;
UPDATE `users` SET `Name` = '李四', `Email` = 'lisi@example.com' WHERE `ID` = 2;
```

---

## 3. UPSERT 模式

生成 UPSERT（如果不存在则插入，如果存在则更新）语句。

### 使用方法

**必须指定主键列**：

```bash
excel-cli convert -i users.xlsx -o users_upsert.sql -f sql \
  --sql-dialect mysql \
  --sql-table users \
  --sql-mode upsert \
  --primary-keys ID
```

### 方言特定语法

不同数据库使用不同的 UPSERT 语法：

#### MySQL / MariaDB

```sql
INSERT INTO `users` (`ID`, `Name`, `Email`) VALUES (1, '张三', 'zhangsan@example.com')
ON DUPLICATE KEY UPDATE `Name` = VALUES(`Name`), `Email` = VALUES(`Email`);
```

#### PostgreSQL

```sql
INSERT INTO "users" ("ID", "Name", "Email") VALUES (1, '张三', 'zhangsan@example.com')
ON CONFLICT ("ID") DO UPDATE SET "Name" = EXCLUDED."Name", "Email" = EXCLUDED."Email";
```

#### SQLite

```sql
INSERT INTO "users" ("ID", "Name", "Email") VALUES (1, '张三', 'zhangsan@example.com')
ON CONFLICT ("ID") DO UPDATE SET "Name" = EXCLUDED."Name", "Email" = EXCLUDED."Email";
```

#### SQL Server

```sql
MERGE INTO [users] AS target
USING (VALUES (1, '张三', 'zhangsan@example.com')) AS source ([ID], [Name], [Email])
ON target.[ID] = source.[ID]
WHEN MATCHED THEN
    UPDATE SET [Name] = source.[Name], [Email] = source.[Email]
WHEN NOT MATCHED THEN
    INSERT ([ID], [Name], [Email]) VALUES (source.[ID], source.[Name], source.[Email]);
```

#### Oracle

```sql
MERGE INTO "users" target
USING (SELECT 1 AS "ID", '张三' AS "Name", 'zhangsan@example.com' AS "Email" FROM DUAL) source
ON (target."ID" = source."ID")
WHEN MATCHED THEN
    UPDATE SET "Name" = source."Name", "Email" = source."Email"
WHEN NOT MATCHED THEN
    INSERT ("ID", "Name", "Email") VALUES (source."ID", source."Name", source."Email");
```

---

## 4. 完整示例

### 示例数据（users.xlsx）

| ID | Name | Email | Age |
|----|------|-------|-----|
| 1 | 张三 | zhangsan@example.com | 25 |
| 2 | 李四 | lisi@example.com | 30 |

### INSERT 模式

```bash
excel-cli convert -i users.xlsx -o insert.sql -f sql \
  --sql-dialect mysql --sql-table users
```

输出：
```sql
INSERT INTO `users` (`ID`, `Name`, `Email`, `Age`) VALUES (1, '张三', 'zhangsan@example.com', 25);
INSERT INTO `users` (`ID`, `Name`, `Email`, `Age`) VALUES (2, '李四', 'lisi@example.com', 30);
```

### UPDATE 模式

```bash
excel-cli convert -i users.xlsx -o update.sql -f sql \
  --sql-dialect mysql --sql-table users \
  --sql-mode update --primary-keys ID
```

输出：
```sql
UPDATE `users` SET `Name` = '张三', `Email` = 'zhangsan@example.com', `Age` = 25 WHERE `ID` = 1;
UPDATE `users` SET `Name` = '李四', `Email` = 'lisi@example.com', `Age` = 30 WHERE `ID` = 2;
```

### UPSERT 模式（MySQL）

```bash
excel-cli convert -i users.xlsx -o upsert.sql -f sql \
  --sql-dialect mysql --sql-table users \
  --sql-mode upsert --primary-keys ID
```

输出：
```sql
INSERT INTO `users` (`ID`, `Name`, `Email`, `Age`) VALUES (1, '张三', 'zhangsan@example.com', 25)
ON DUPLICATE KEY UPDATE `Name` = VALUES(`Name`), `Email` = VALUES(`Email`), `Age` = VALUES(`Age`);
```

---

## 5. 注意事项

### UPDATE 和 UPSERT 模式

- **必须指定主键**：使用 `--primary-keys` 参数
- **主键自动排除**：在 UPDATE 和 UPSERT 模式中，主键列不会出现在 SET 子句中
- **多个主键**：使用逗号分隔，例如 `--primary-keys ID,Email`

### 指定更新列

- 仅对 UPDATE 和 UPSERT 模式有效
- 使用 `--update-columns` 参数
- 示例：`--update-columns Name,Email,Age`

### 列名映射

可以与 `--column-mapping` 参数组合使用：

```bash
excel-cli convert -i users.xlsx -o users.sql -f sql \
  --sql-dialect mysql --sql-table users \
  --sql-mode update \
  --primary-keys user_id \
  --column-mapping user_id,user_name,user_email,user_age
```

---

## 6. 支持的数据库方言

- MySQL / MariaDB (`mysql` / `mariadb`)
- PostgreSQL (`postgresql` / `postgres` / `pg`)
- SQLite (`sqlite` / `sqlite3`)
- SQL Server (`sqlserver` / `mssql` / `tsql`)
- Oracle (`oracle`)

每个方言会生成相应的正确语法。

---

## 7. 常见用例

### 初始化数据库表

```bash
# 1. 生成建表语句
excel-cli schema -i users.xlsx -o schema.sql \
  --sql-dialect mysql --sql-table users --primary-key ID

# 2. 生成插入语句
excel-cli convert -i users.xlsx -o data.sql -f sql \
  --sql-dialect mysql --sql-table users
```

### 批量更新现有记录

```bash
excel-cli convert -i users_updated.xlsx -o update.sql -f sql \
  --sql-dialect mysql --sql-table users \
  --sql-mode update --primary-keys ID
```

### 同步数据（存在则更新，不存在则插入）

```bash
excel-cli convert -i users_sync.xlsx -o upsert.sql -f sql \
  --sql-dialect postgresql --sql-table users \
  --sql-mode upsert --primary-keys ID
```

---

## 8. 错误处理

### 未指定主键

```bash
$ excel-cli convert -i users.xlsx -o update.sql -f sql --sql-mode update
❌ 错误: UPDATE 模式需要指定主键列 (--primary-keys)
```

### 主键列不存在

```bash
$ excel-cli convert -i users.xlsx -o update.sql -f sql \
  --sql-mode update --primary-keys invalid_column
❌ 错误: 主键列 'invalid_column' 在 Excel 中不存在
可用列: ID, Name, Email, Age
```

---

## 9. 性能建议

- 对于大量数据，建议使用数据库原生的批量加载工具
- 使用事务包裹多个语句
- 考虑使用批量 INSERT 语句（未来版本可能支持）

---

## 10. 下一步

- 查看[模板导出指南](TEMPLATE_GUIDE.md)了解 HTML、Markdown、XML、YAML 导出
- 查看[主 README](../README.md)了解更多功能
