# bm — Chrome 书签管理 CLI

Chrome 书签管理命令行工具，支持查看结构、搜索、去重、分析、死链检测以及写操作（删除/移动/重命名/排序/文件夹管理）。

## 安装

```bash
cargo build --release
# 二进制在 target/release/bm
```

或直接使用预构建版本：

```bash
./skills/bookmarks/scripts/bm
```

## 用法

```
bm [--profile N] <子命令>
```

`--profile N` 指定 Chrome Profile 编号（如 `--profile 1` 对应 `Profile 1`）。省略时自动选取第一个可用 Profile。

---

## 子命令

### 只读操作

| 子命令 | 功能 |
|--------|------|
| `structure` | 展示书签文件夹树（默认命令） |
| `dupes` | 查找重复书签（相同 URL） |
| `analyze` | 分析文件夹结构问题，给出优化建议 |
| `search <关键词>` | 模糊搜索书签（名称 + URL），按相关度排序 |
| `stats [--top N]` | 统计书签数量分布（按文件夹 / 按域名 Top N，默认 20） |
| `deadlinks` | 并发检测死链，见下方详细说明 |

### 写操作

> **安全规则：所有写操作执行前必须完全关闭 Chrome，否则 Chrome 重启后会覆盖修改。**

| 子命令 | 功能 |
|--------|------|
| `delete <关键词> [--dry-run]` | 删除匹配书签（名称或 URL 包含关键词） |
| `mv <关键词> --to <文件夹路径> [--dry-run]` | 移动匹配书签到指定文件夹 |
| `rename <关键词> --name <新名称> [--dry-run]` | 重命名匹配书签 |
| `sort <文件夹路径> [--dry-run]` | 对指定文件夹内书签按名称排序 |
| `mkdir <完整路径> [--dry-run]` | 新建文件夹（支持多级路径） |
| `mvdir <源路径> --to <目标父文件夹> [--dry-run]` | 移动文件夹到指定位置 |

---

## 示例

```bash
# 查看书签树
bm structure

# 搜索包含 "github" 的书签
bm search github

# 查看书签统计（Top 30 域名）
bm stats --top 30

# 检测死链（200 并发，10s 超时）
bm deadlinks --concurrency 200 --timeout 10

# 预览删除含 "bilibili" 的书签（不实际修改）
bm delete bilibili --dry-run

# 确认后执行删除
bm delete bilibili

# 移动书签到指定文件夹（先预览）
bm mv "云服务器" --to "书签栏/技术开发/服务器" --dry-run
bm mv "云服务器" --to "书签栏/技术开发/服务器"

# 新建多级文件夹
bm mkdir "书签栏/技术开发/新工具" --dry-run
bm mkdir "书签栏/技术开发/新工具"

# 移动文件夹
bm mvdir "书签栏/旧分类" --to "书签栏/归档" --dry-run

# 对文件夹内书签排序（文件夹优先，各组内字典序）
bm sort "书签栏/工具" --dry-run
```

---

## 写操作安全机制

- **`--dry-run`**：预览变更，不修改文件，建议每次写操作前先执行
- **自动备份**：每次写操作前自动创建 `Bookmarks.bm-<毫秒时间戳>` 备份文件
- **文件夹路径格式**：`书签栏/云服务器/网络监控`（用 `/` 分隔层级）

---

## 死链检测说明

```bash
bm deadlinks [--concurrency 100] [--timeout 8]
```

- 自动去重：相同 URL 只请求一次
- 输出分为四类：
  - HTTP 错误（4xx / 5xx）
  - 超时
  - 连接失败
  - 已重定向（仍可访问，URL 已变）

---

## 依赖

- Rust 2021 edition
- macOS（书签路径基于 `~/Library/Application Support/Google/Chrome`）
