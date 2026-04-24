---
name: bookmarks
description: >
  浏览器书签管理工具（Chrome）。支持查看书签结构、查找重复书签、分析优化建议、搜索书签、并发检测死链。
  触发时机：用户提到「书签」「bookmark」「整理书签」「查重复书签」「书签搜索」「死链」「失效书签」等关键词时。
license: MIT
metadata:
  author: c.chen
  version: "2.0.0"
---

# bookmarks

二进制：`{BASE_DIR}/scripts/bm`

> `{BASE_DIR}` 由 Claude Code 的 `Base directory for this skill` 提供，调用时替换为实际路径。

## 调用格式

```bash
{BASE_DIR}/scripts/bm [--profile N] <子命令>
```

`--profile 1` 指定 Chrome Profile 编号（默认自动选 Profile 1）。

## 子命令对照表

| 用户意图 | 子命令 |
|---------|--------|
| 查看书签结构/文件夹层级 | `structure` |
| 查找/清理重复书签 | `dupes` |
| 分析书签组织是否合理、给优化建议 | `analyze` |
| 搜索特定书签 | `search <关键词>` |
| 检测死链/失效书签 | `deadlinks [--concurrency 100] [--timeout 8]` |

## 死链检测说明

- 自动去重（相同 URL 只请求一次）
- 并发默认 100，超时默认 8s，可调节
- 输出分类：HTTP 错误 / 超时 / 连接失败 / 已重定向（仍可访问）

## 执行流程

1. 判断用户意图，选对子命令
2. 用 `Base directory for this skill` 拼出完整路径，运行命令
3. 基于输出给出中文分析和建议

## NEVER

- **NEVER 直接修改书签文件**：如用户要求执行修改，必须先提醒「请先完全关闭浏览器」并做备份
- **NEVER 未经确认就重组或删除书签**
