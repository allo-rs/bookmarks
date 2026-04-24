---
name: bookmarks
description: >
  浏览器书签管理工具。支持查看书签结构、查找重复书签、分析优化建议、搜索书签。
  触发时机：用户提到「书签」「bookmark」「整理书签」「查重复书签」「书签搜索」等关键词时。
license: MIT
metadata:
  author: c.chen
  version: "1.0.0"
---

# bookmarks

脚本：`~/dev/bookmarks/scripts/bookmarks.py`

## 调用格式

```bash
python3 ~/dev/bookmarks/scripts/bookmarks.py [--browser chrome|edge] [--profile N] <子命令>
```

浏览器参数可省略（自动检测）。`--profile 1` 指定 Chrome Profile 编号。

## 子命令对照表

| 用户意图 | 子命令 |
|---------|--------|
| 查看书签结构/文件夹层级 | `structure` |
| 查找/清理重复书签 | `dupes` |
| 分析书签组织是否合理、给优化建议 | `analyze` |
| 搜索特定书签 | `search <关键词>` |

## 执行流程

1. 判断用户意图，选对子命令
2. 运行脚本，获取输出
3. 基于输出给出中文分析和建议

## NEVER

- **NEVER 直接修改书签文件**：如用户要求执行修改，必须先提醒「请先完全关闭浏览器」并做备份
- **NEVER 未经确认就重组或删除书签**
