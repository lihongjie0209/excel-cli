#!/usr/bin/env python3
"""创建测试用的 Excel 文件"""

import openpyxl
from datetime import datetime

# 创建工作簿
wb = openpyxl.Workbook()
ws = wb.active
ws.title = "users"

# 添加表头
headers = ["ID", "Name", "Email", "Age", "City", "Registered"]
ws.append(headers)

# 添加测试数据
test_data = [
    [1, "张三", "zhangsan@example.com", 25, "北京", datetime(2023, 1, 15).strftime("%Y-%m-%d")],
    [2, "李四", "lisi@example.com", 30, "上海", datetime(2023, 2, 20).strftime("%Y-%m-%d")],
    [3, "王五", "wangwu@example.com", 28, "深圳", datetime(2023, 3, 10).strftime("%Y-%m-%d")],
    [4, "赵六", "zhaoliu@example.com", 35, "广州", datetime(2023, 4, 5).strftime("%Y-%m-%d")],
    [5, "钱七", "qianqi@example.com", 22, "杭州", datetime(2023, 5, 12).strftime("%Y-%m-%d")],
]

for row in test_data:
    ws.append(row)

# 保存文件
wb.save("test_users.xlsx")
print("✅ 测试文件已创建: test_users.xlsx")
