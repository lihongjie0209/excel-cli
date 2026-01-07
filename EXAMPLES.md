# Excel CLI 使用示例

## 快速开始

### 1. 查看支持的格式

```bash
excel-cli formats
```

输出：
```
📦 支持的导出格式:
  • json
  • csv

💡 提示: 可以通过实现 Exporter trait 添加更多格式支持
```

### 2. 转换 Excel 到 JSON

```bash
excel-cli convert -i sample.xlsx -o output.json -f json
```

输出：
```
📖 正在读取 Excel 文件: sample.xlsx
✅ 成功读取工作表 'Sheet1': 100 行 × 5 列
📝 正在导出为 JSON 格式...
✅ 转换完成! 输出文件: output.json
```

### 3. 转换 Excel 到 CSV

```bash
excel-cli convert -i sample.xlsx -o output.csv -f csv
```

### 4. 指定特定工作表

```bash
excel-cli convert -i sample.xlsx -o output.json -f json -s "Sales Data"
```

### 5. 列出所有工作表

```bash
excel-cli list-sheets -i sample.xlsx
```

输出：
```
📋 工作表列表:
  1. Sheet1
  2. Sales Data
  3. Inventory
```

## 批量处理示例

### 转换当前目录下所有 Excel 文件为 JSON

**PowerShell:**
```powershell
Get-ChildItem *.xlsx | ForEach-Object {
    $name = $_.BaseName
    excel-cli convert -i $_.Name -o "$name.json" -f json
}
```

**Bash (Linux/Mac):**
```bash
for file in *.xlsx; do
    name="${file%.xlsx}"
    excel-cli convert -i "$file" -o "$name.json" -f json
done
```

### 转换特定工作表到多个格式

```bash
# 转换为 JSON
excel-cli convert -i data.xlsx -o sales.json -f json -s "Sales"

# 转换为 CSV
excel-cli convert -i data.xlsx -o sales.csv -f csv -s "Sales"
```

## 输出示例

### JSON 输出格式

输入 Excel:
| Name  | Age | City     |
|-------|-----|----------|
| Alice | 30  | Beijing  |
| Bob   | 25  | Shanghai |

输出 JSON:
```json
[
  {
    "Name": "Alice",
    "Age": 30.0,
    "City": "Beijing"
  },
  {
    "Name": "Bob",
    "Age": 25.0,
    "City": "Shanghai"
  }
]
```

### CSV 输出格式

```csv
Name,Age,City
Alice,30,Beijing
Bob,25,Shanghai
```

## 错误处理

### 文件不存在

```bash
excel-cli convert -i nonexistent.xlsx -o output.json
```

输出：
```
❌ 错误: 输入文件不存在: nonexistent.xlsx
```

### 不支持的格式

```bash
excel-cli convert -i data.xlsx -o output.xml -f xml
```

输出：
```
Error: 不支持的导出格式: xml
```

### 工作表不存在

```bash
excel-cli convert -i data.xlsx -o output.json -s "NonExistent"
```

输出：
```
Error: 工作表 'NonExistent' 不存在
```

## 高级用法

### 管道处理

使用 PowerShell 管道处理：

```powershell
# 列出工作表并选择
$sheets = excel-cli list-sheets -i data.xlsx
# 然后手动选择要转换的工作表
```

### 集成到脚本

```powershell
# data-processor.ps1
param(
    [string]$InputDir = ".",
    [string]$OutputDir = "./converted"
)

# 创建输出目录
New-Item -ItemType Directory -Force -Path $OutputDir | Out-Null

# 转换所有 Excel 文件
Get-ChildItem -Path $InputDir -Filter "*.xlsx" | ForEach-Object {
    $outputFile = Join-Path $OutputDir "$($_.BaseName).json"
    Write-Host "Processing: $($_.Name)"
    excel-cli convert -i $_.FullName -o $outputFile -f json
}

Write-Host "✅ 所有文件处理完成!"
```

## 性能提示

- 对于大型 Excel 文件（数万行），JSON 格式可能会生成较大的文件
- CSV 格式通常更紧凑，适合大数据量
- 处理多个文件时，建议使用批处理脚本提高效率

## 故障排除

### 问题：无法读取 Excel 文件

**可能原因：**
- 文件正在被其他程序打开（如 Excel）
- 文件损坏或格式不正确
- 权限不足

**解决方法：**
- 关闭所有打开该文件的程序
- 尝试重新保存 Excel 文件
- 检查文件权限

### 问题：输出文件为空或不完整

**可能原因：**
- 工作表为空或只有表头
- Excel 文件包含特殊格式或公式

**解决方法：**
- 使用 `list-sheets` 检查工作表内容
- 确保 Excel 文件包含数据（不只是格式）
- 尝试在 Excel 中"另存为"新文件

## 更多信息

访问项目仓库获取更多帮助：
https://github.com/yourusername/excel-cli
