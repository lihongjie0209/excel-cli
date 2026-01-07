import openpyxl
from datetime import datetime

# 创建工作簿
wb = openpyxl.Workbook()
ws = wb.active
ws.title = "Users"

# 添加表头
headers = ["ID", "Name", "Age", "City", "Salary", "Active", "JoinDate"]
ws.append(headers)

# 添加数据
data = [
    [1, "张三", 28, "北京", 15000, True, "2020-01-15"],
    [2, "李四", 35, "上海", 22000, True, "2019-06-20"],
    [3, "王五", 42, "广州", 18000, False, "2018-03-10"],
    [4, "赵六", 31, "深圳", 25000, True, "2021-09-05"],
    [5, "钱七", 29, "北京", 16000, True, "2020-11-12"],
    [6, "孙八", 45, "上海", 30000, True, "2017-08-22"],
    [7, "周九", 26, "广州", 12000, False, "2022-02-14"],
    [8, "吴十", 38, "深圳", 28000, True, "2019-12-01"],
]

for row in data:
    ws.append(row)

# 保存文件
wb.save("test_data.xlsx")
print("✅ 测试数据文件已生成: test_data.xlsx")
