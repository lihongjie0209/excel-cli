# 模板导出使用指南

excel-cli 支持使用模板引擎（Tera）将 Excel 数据导出为各种自定义格式。

## 1. 内置模板格式

excel-cli 提供了 4 种内置模板格式：

| 格式 | 命令参数 | 文件扩展名 | 说明 |
|------|----------|-----------|------|
| HTML 表格 | `html` / `html-table` | `.html` | 完整的 HTML 页面，带样式 |
| Markdown 表格 | `markdown` / `md` / `md-table` | `.md` | Markdown 格式表格 |
| XML | `xml` | `.xml` | 标准 XML 结构 |
| YAML | `yaml` / `yml` | `.yaml` | YAML 格式 |

---

## 2. HTML 表格导出

### 使用方法

```bash
excel-cli convert -i users.xlsx -o users.html -f html
```

或使用完整格式名：

```bash
excel-cli convert -i users.xlsx -o users.html -f html-table
```

### 生成效果

生成的 HTML 文件包含：
- 完整的 HTML5 文档结构
- 内嵌 CSS 样式（表格样式、悬停效果）
- 响应式设计
- 元数据展示（工作表名、行数、列数）

### 示例输出

```html
<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>users</title>
    <style>
        body {
            font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
            padding: 20px;
            background-color: #f5f5f5;
        }
        table {
            border-collapse: collapse;
            width: 100%;
            background-color: white;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }
        th, td {
            border: 1px solid #ddd;
            padding: 12px;
            text-align: left;
        }
        th {
            background-color: #4CAF50;
            color: white;
        }
        tr:hover {
            background-color: #f5f5f5;
        }
    </style>
</head>
<body>
    <h1>users</h1>
    <p>总行数: 5 | 总列数: 6</p>
    <table>
        <thead>
            <tr>
                <th>ID</th>
                <th>Name</th>
                <th>Email</th>
                <!-- ... -->
            </tr>
        </thead>
        <tbody>
            <tr>
                <td>1</td>
                <td>张三</td>
                <td>zhangsan@example.com</td>
                <!-- ... -->
            </tr>
            <!-- ... -->
        </tbody>
    </table>
</body>
</html>
```

可以直接在浏览器中打开查看。

---

## 3. Markdown 表格导出

### 使用方法

```bash
excel-cli convert -i users.xlsx -o users.md -f markdown
```

或使用简写：

```bash
excel-cli convert -i users.xlsx -o users.md -f md
```

### 生成效果

生成标准的 Markdown 表格，兼容 GitHub、GitLab、各种 Markdown 编辑器。

### 示例输出

```markdown
# users

**总行数**: 5 | **总列数**: 6

| ID | Name | Email | Age | City | Registered |
| --- | --- | --- | --- | --- | --- |
| 1 | 张三 | zhangsan@example.com | 25 | 北京 | 2023-01-15 |
| 2 | 李四 | lisi@example.com | 30 | 上海 | 2023-02-20 |
| 3 | 王五 | wangwu@example.com | 28 | 深圳 | 2023-03-10 |
```

---

## 4. XML 导出

### 使用方法

```bash
excel-cli convert -i users.xlsx -o users.xml -f xml
```

### 生成效果

生成结构化的 XML 文档，包含元数据和数据行。

### 示例输出

```xml
<?xml version="1.0" encoding="UTF-8"?>
<data>
    <meta>
        <sheet_name>users</sheet_name>
        <row_count>5</row_count>
        <column_count>6</column_count>
    </meta>
    <headers>
        <header>ID</header>
        <header>Name</header>
        <header>Email</header>
        <!-- ... -->
    </headers>
    <rows>
        <row>
            <ID>1</ID>
            <Name>张三</Name>
            <Email>zhangsan@example.com</Email>
            <!-- ... -->
        </row>
        <!-- ... -->
    </rows>
</data>
```

---

## 5. YAML 导出

### 使用方法

```bash
excel-cli convert -i users.xlsx -o users.yaml -f yaml
```

或使用 `.yml` 扩展名：

```bash
excel-cli convert -i users.xlsx -o users.yml -f yml
```

### 生成效果

生成 YAML 格式的数据，适合配置文件或数据交换。

### 示例输出

```yaml
sheet_name: users
row_count: 5
column_count: 6
rows:
- 
    ID: 1
    Name: 张三
    Email: zhangsan@example.com
    Age: 25
    City: 北京
    Registered: 2023-01-15
- 
    ID: 2
    Name: 李四
    Email: lisi@example.com
    Age: 30
    City: 上海
    Registered: 2023-02-20
```

---

## 6. 自定义模板

### 创建自定义模板

excel-cli 使用 [Tera](https://tera.netlify.app/) 模板引擎（语法类似 Jinja2）。

#### 模板变量

在模板中可以访问以下变量：

| 变量 | 类型 | 说明 |
|------|------|------|
| `sheet_name` | String | 工作表名称 |
| `row_count` | Number | 数据行数 |
| `column_count` | Number | 列数 |
| `headers` | Array[String] | 表头列表 |
| `rows` | Array[Object] | 数据行，每行是一个对象 |

#### 示例：创建 CSV 模板

创建文件 `my_template.tera`：

```tera
{# CSV 导出模板 #}
{% for header in headers %}{{ header }}{% if not loop.last %},{% endif %}{% endfor %}
{% for row in rows -%}
{% for header in headers -%}
{{ row[header] | default(value="") }}{% if not loop.last %},{% endif %}
{%- endfor %}
{% endfor -%}
```

#### 示例：创建 JSON 模板

创建文件 `json_template.tera`：

```tera
{
  "sheet_name": "{{ sheet_name }}",
  "row_count": {{ row_count }},
  "column_count": {{ column_count }},
  "headers": [
    {% for header in headers %}"{{ header }}"{% if not loop.last %}, {% endif %}{% endfor %}
  ],
  "data": [
    {% for row in rows -%}
    {
      {% for header in headers -%}
      "{{ header }}": {{ row[header] | json_encode() }}{% if not loop.last %}, {% endif %}
      {%- endfor %}
    }{% if not loop.last %}, {% endif %}
    {%- endfor %}
  ]
}
```

#### 示例：创建自定义 HTML 报告

创建文件 `report.tera`：

```html
<!DOCTYPE html>
<html>
<head>
    <title>{{ sheet_name }} 报告</title>
    <style>
        body { font-family: Arial; }
        .summary { background: #f0f0f0; padding: 10px; }
        table { border-collapse: collapse; margin-top: 20px; }
        th { background: #333; color: white; padding: 8px; }
        td { border: 1px solid #ccc; padding: 8px; }
        .even { background: #f9f9f9; }
    </style>
</head>
<body>
    <h1>{{ sheet_name }} 数据报告</h1>
    <div class="summary">
        <p>数据统计：</p>
        <ul>
            <li>总行数：{{ row_count }}</li>
            <li>总列数：{{ column_count }}</li>
        </ul>
    </div>
    <table>
        <thead>
            <tr>
                {% for header in headers %}<th>{{ header }}</th>{% endfor %}
            </tr>
        </thead>
        <tbody>
            {% for row in rows %}
            <tr class="{% if loop.index0 is even %}even{% endif %}">
                {% for header in headers %}
                <td>{{ row[header] }}</td>
                {% endfor %}
            </tr>
            {% endfor %}
        </tbody>
    </table>
</body>
</html>
```

### 使用自定义模板

```bash
excel-cli convert -i users.xlsx -o output.txt -f template --template my_template.tera
```

或使用绝对路径：

```bash
excel-cli convert -i users.xlsx -o output.html -f template \
  --template /path/to/report.tera
```

---

## 7. Tera 模板语法速查

### 变量输出

```tera
{{ variable_name }}
{{ row["Column Name"] }}
```

### 循环

```tera
{% for item in items %}
  {{ item }}
{% endfor %}

{# 带索引 #}
{% for row in rows %}
  行号: {{ loop.index }} {# 从 1 开始 #}
  索引: {{ loop.index0 }} {# 从 0 开始 #}
{% endfor %}
```

### 条件

```tera
{% if condition %}
  内容
{% elif other_condition %}
  其他内容
{% else %}
  默认内容
{% endif %}
```

### 过滤器

```tera
{{ text | upper }}          {# 大写 #}
{{ text | lower }}          {# 小写 #}
{{ text | trim }}           {# 去空格 #}
{{ value | default(value="N/A") }}  {# 默认值 #}
{{ data | json_encode() }}  {# JSON 编码 #}
```

### 注释

```tera
{# 这是注释 #}
```

---

## 8. 完整示例

### 场景：导出用户数据为不同格式

假设有 Excel 文件 `users.xlsx`：

| ID | Name | Email | Department |
|----|------|-------|------------|
| 1 | 张三 | zhang@example.com | 技术部 |
| 2 | 李四 | li@example.com | 销售部 |

### HTML 报告

```bash
excel-cli convert -i users.xlsx -o users_report.html -f html
```

### Markdown 文档

```bash
excel-cli convert -i users.xlsx -o USERS.md -f markdown
```

### XML 数据交换

```bash
excel-cli convert -i users.xlsx -o users_data.xml -f xml
```

### YAML 配置

```bash
excel-cli convert -i users.xlsx -o users.yaml -f yaml
```

### 自定义 LaTeX 表格

创建 `latex_table.tera`：

```tera
\begin{table}[h]
\centering
\caption{ {{ sheet_name }} }
\begin{tabular}{ |{% for header in headers %}c|{% endfor %} }
\hline
{% for header in headers %}{{ header }}{% if not loop.last %} & {% endif %}{% endfor %} \\
\hline
{% for row in rows -%}
{% for header in headers -%}
{{ row[header] }}{% if not loop.last %} & {% endif %}
{%- endfor %} \\
{% endfor -%}
\hline
\end{tabular}
\end{table}
```

使用：

```bash
excel-cli convert -i users.xlsx -o users_table.tex -f template \
  --template latex_table.tera
```

---

## 9. 与数据过滤组合

模板导出可以与数据过滤功能组合使用：

### 只导出特定列

```bash
excel-cli convert -i users.xlsx -o filtered.html -f html \
  --select ID,Name,Email
```

### 排除敏感列

```bash
excel-cli convert -i users.xlsx -o public.md -f markdown \
  --exclude Password,SSN
```

### 带条件过滤

```bash
excel-cli convert -i users.xlsx -o active_users.html -f html \
  --filter "Status=active"
```

---

## 10. 注意事项

### 字符编码

- 所有模板文件必须使用 UTF-8 编码
- 生成的文件也是 UTF-8 编码

### 特殊字符处理

- HTML 模板会自动转义 HTML 特殊字符
- XML 模板会自动转义 XML 特殊字符
- 其他格式根据需要手动处理

### 性能

- 内置模板已优化性能
- 自定义模板注意避免复杂的嵌套循环
- 对于大量数据（>10000 行），考虑分批处理

### 模板调试

使用简单的测试数据：

```bash
# 创建小型测试文件
excel-cli convert -i users.xlsx -o test.txt -f template \
  --template my_template.tera \
  --filter "ID<=10"
```

---

## 11. 内置模板源码

所有内置模板位于项目的 `templates/` 目录：

```
templates/
├── html_table.tera       # HTML 表格模板
├── markdown_table.tera   # Markdown 表格模板
├── xml.tera              # XML 格式模板
└── yaml.tera             # YAML 格式模板
```

可以参考这些模板创建自己的模板。

---

## 12. 故障排除

### 模板文件找不到

```bash
❌ 错误: 无法读取模板文件: No such file or directory
```

解决方法：
- 检查文件路径是否正确
- 使用绝对路径
- 检查文件权限

### 模板语法错误

```bash
❌ 错误: 模板渲染失败: unexpected token
```

解决方法：
- 检查 Tera 语法是否正确
- 检查 `{% %}` 和 `{{ }}` 是否配对
- 参考 [Tera 官方文档](https://tera.netlify.app/docs/)

### 变量不存在

```bash
❌ 警告: Variable `unknown_var` not found
```

解决方法：
- 确认使用的变量名正确
- 使用 `| default(value="")` 提供默认值

---

## 13. 下一步

- 查看 [UPDATE/UPSERT 指南](UPDATE_UPSERT_GUIDE.md) 了解 SQL 生成功能
- 查看 [主 README](../README.md) 了解更多功能
- 访问 [Tera 文档](https://tera.netlify.app/) 学习高级模板技巧
