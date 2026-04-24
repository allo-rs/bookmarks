---
name: bookmarks
description: >
  浏览器书签管理工具（Chrome）。支持查看书签结构、查找重复书签、分析优化建议、搜索书签、并发检测死链、删除和移动书签。
  触发时机：用户提到「书签」「bookmark」「整理书签」「查重复书签」「书签搜索」「死链」「失效书签」「删除书签」「移动书签」等关键词时。
license: MIT
metadata:
  author: c.chen
  version: "2.1.0"
---

# bookmarks

## 调用格式

```bash
./scripts/bm [--profile N] <子命令>
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
| 删除书签 | `delete <关键词> [--dry-run]` |
| 移动书签到指定文件夹 | `mv <关键词> --to <文件夹路径> [--dry-run]` |

## 写操作说明（delete / mv）

- **必须先 `--dry-run` 预览**，确认无误后再去掉该参数执行
- 执行前自动备份书签文件（`Bookmarks.bm-<时间戳>`）
- `<文件夹路径>` 格式：`书签栏/云服务器/网络监控`
- **执行写操作前必须完全关闭 Chrome**，否则 Chrome 重启后会覆盖修改

## 死链检测说明

- 自动去重（相同 URL 只请求一次）
- 并发默认 100，超时默认 8s，可调节
- 输出分类：HTTP 错误 / 超时 / 连接失败 / 已重定向（仍可访问）

## 执行流程

1. 判断用户意图，选对子命令
2. 写操作先 `--dry-run` 给用户确认，用户确认后再执行
3. 运行命令，获取输出
4. 基于输出给出中文分析和建议

## NEVER

- **NEVER 未经 --dry-run 确认直接执行 delete / mv**
- **NEVER 在 Chrome 未完全关闭时执行写操作**
- **NEVER 未经用户确认就执行删除或移动**
