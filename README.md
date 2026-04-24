# bookmarks — Chrome 书签管理 Skill

这是一个 [Claude Code Skill](https://docs.claude.ai/skills)，让 Claude 能够直接管理你的 Chrome 书签。

对话中提到书签相关需求时，Claude 会自动调用本 skill，无需手动输入命令。

## 工作原理

```
用户说「帮我找重复书签」
    ↓
Claude 触发 bookmarks skill
    ↓
skill 调用 scripts/bm dupes（Rust 编译的本地二进制）
    ↓
Claude 解读输出，给出中文分析和建议
```

底层工具是 `bm`，一个用 Rust 编写的 Chrome 书签 CLI，直接读写 `~/Library/Application Support/Google/Chrome/*/Bookmarks`。

---

## 安装 Skill

```bash
# 1. 编译二进制
cargo build --release
cp target/release/bm skills/bookmarks/scripts/bm

# 2. 部署到 Claude Code
cp -r skills/bookmarks ~/.claude/skills/bookmarks
```

之后重启 Claude Code 即可生效。

---

## 触发关键词

用户消息中包含以下关键词时 skill 自动激活：

`书签` `bookmark` `整理书签` `查重复书签` `书签搜索` `死链` `失效书签`  
`删除书签` `移动书签` `重命名书签` `书签统计` `排序书签` `新建文件夹` `移动文件夹`

---

## 支持的操作

### 只读

| 操作 | 触发示例 |
|------|---------|
| 查看书签树结构 | 「显示我的书签文件夹」 |
| 查找重复书签 | 「有没有重复的书签」 |
| 分析结构问题 | 「帮我分析一下书签组织」 |
| 模糊搜索 | 「搜索 github 相关书签」 |
| 统计分布 | 「统计各文件夹书签数量」 |
| 检测死链 | 「检测失效书签」 |

### 写操作（执行前必须关闭 Chrome）

| 操作 | 触发示例 |
|------|---------|
| 删除书签 | 「删掉 bilibili 相关书签」 |
| 移动书签 | 「把云服务器书签移到技术开发文件夹」 |
| 重命名书签 | 「把这个书签改名为 XX」 |
| 排序文件夹 | 「把工具文件夹里的书签排个序」 |
| 新建文件夹 | 「在书签栏下新建一个"归档"文件夹」 |
| 移动文件夹 | 「把旧分类文件夹移到归档下面」 |

> 所有写操作均有 `--dry-run` 预览步骤，Claude 会先让你确认再执行。  
> 每次写操作前自动备份书签文件（`Bookmarks.bm-<时间戳>`）。

---

## 直接使用 CLI

不经过 skill，也可以直接运行：

```bash
bm [--profile N] <子命令>
```

```bash
bm structure                          # 查看书签树
bm search github                      # 搜索书签
bm dupes                              # 查重复
bm stats --top 30                     # 统计 Top 30 域名
bm deadlinks --concurrency 200        # 检测死链
bm delete bilibili --dry-run          # 预览删除
bm mv "云服务器" --to "书签栏/技术" --dry-run
bm mkdir "书签栏/归档/2024" --dry-run
bm mvdir "书签栏/旧分类" --to "书签栏/归档" --dry-run
bm sort "书签栏/工具" --dry-run
```

`--profile N`：指定 Chrome Profile 编号，省略时自动选取第一个。

---

## 项目结构

```
.
├── src/                  # Rust 源码
│   ├── main.rs           # CLI 入口（clap 子命令定义）
│   ├── bookmark.rs       # 书签数据结构与解析
│   ├── finder.rs         # Chrome 书签文件定位
│   ├── write.rs          # 写操作（备份 + 保存）
│   └── cmd/              # 各子命令实现
├── skills/bookmarks/     # Claude Code Skill 包
│   ├── SKILL.md          # Skill 定义（Claude 读取此文件）
│   └── scripts/bm        # 预编译二进制
└── Cargo.toml
```

## 依赖

- Rust 2021 edition
- macOS（书签路径基于 `~/Library/Application Support/Google/Chrome`）
