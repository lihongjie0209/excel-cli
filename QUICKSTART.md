# 快速入门指南

## 安装

### 方式 1: 从源码构建

```bash
git clone <your-repo-url>
cd excel-cli
cargo build --release
```

可执行文件位于：`target/release/excel-cli.exe` (Windows) 或 `target/release/excel-cli` (Linux/Mac)

### 方式 2: 直接安装到系统

```bash
cargo install --path .
```

安装后可在任何位置使用 `excel-cli` 命令。

## 基本使用

### 1. 转换 Excel 到 JSON

```bash
excel-cli convert -i data.xlsx -o output.json
```

### 2. 转换 Excel 到 CSV

```bash
excel-cli convert -i data.xlsx -o output.csv -f csv
```

### 3. 指定工作表

```bash
excel-cli convert -i data.xlsx -o output.json -s "Sheet2"
```

### 4. 查看所有工作表

```bash
excel-cli list-sheets -i data.xlsx
```

### 5. 查看支持的格式

```bash
excel-cli formats
```

## 命令参数说明

### convert 命令

| 参数 | 简写 | 必填 | 说明 | 默认值 |
|------|------|------|------|--------|
| --input | -i | ✅ | Excel 文件路径 | - |
| --output | -o | ✅ | 输出文件路径 | - |
| --format | -f | ❌ | 输出格式 (json/csv) | json |
| --sheet | -s | ❌ | 工作表名称 | 第一个工作表 |

### list-sheets 命令

| 参数 | 简写 | 必填 | 说明 |
|------|------|------|------|
| --input | -i | ✅ | Excel 文件路径 |

## 输出示例

### Excel 数据

| 姓名 | 年龄 | 城市 |
|------|------|------|
| 张三 | 30 | 北京 |
| 李四 | 25 | 上海 |

### JSON 输出

```json
[
  {
    "姓名": "张三",
    "年龄": 30.0,
    "城市": "北京"
  },
  {
    "姓名": "李四",
    "年龄": 25.0,
    "城市": "上海"
  }
]
```

### CSV 输出

```csv
姓名,年龄,城市
张三,30,北京
李四,25,上海
```

## 常见问题

**Q: 如何批量转换多个 Excel 文件？**

A: 使用 PowerShell 或 Bash 脚本：

```powershell
# PowerShell
Get-ChildItem *.xlsx | ForEach-Object {
    excel-cli convert -i $_.Name -o "$($_.BaseName).json"
}
```

```bash
# Bash
for file in *.xlsx; do
    excel-cli convert -i "$file" -o "${file%.xlsx}.json"
done
```

**Q: 支持哪些 Excel 格式？**

A: 支持 .xlsx 格式（Excel 2007 及更高版本）。

**Q: 如何添加新的导出格式？**

A: 参见 README.md 的"扩展新格式"章节。

## 更多信息

- 完整文档：[README.md](README.md)
- 使用示例：[EXAMPLES.md](EXAMPLES.md)
- 问题反馈：[GitHub Issues](https://github.com/yourusername/excel-cli/issues)
