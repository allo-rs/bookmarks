use clap::{Parser, Subcommand};
use anyhow::Result;

mod bookmark;
mod cmd;
mod finder;
mod write;

#[derive(Parser)]
#[command(name = "bm", about = "Chrome 书签管理工具")]
struct Cli {
    /// Chrome Profile 编号（如 1 对应 Profile 1）
    #[arg(long)]
    profile: Option<String>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// 展示文件夹树
    Structure,
    /// 查找重复书签
    Dupes,
    /// 分析文件夹结构问题
    Analyze,
    /// 搜索书签（模糊匹配 + 相关度排序）
    Search { query: Vec<String> },
    /// 并发检测死链
    Deadlinks {
        #[arg(long, default_value = "100")]
        concurrency: usize,
        #[arg(long, default_value = "8")]
        timeout: u64,
    },
    /// 删除匹配书签（⚠️ 写操作，建议先加 --dry-run）
    Delete {
        /// 关键词（匹配书签名或 URL）
        keyword: Vec<String>,
        /// 预览变更，不实际修改
        #[arg(long)]
        dry_run: bool,
    },
    /// 移动书签到指定文件夹（⚠️ 写操作，建议先加 --dry-run）
    Mv {
        /// 关键词（匹配书签名或 URL）
        keyword: Vec<String>,
        /// 目标文件夹路径，如 "书签栏/云服务器/网络监控"
        #[arg(long)]
        to: String,
        /// 预览变更，不实际修改
        #[arg(long)]
        dry_run: bool,
    },
    /// 重命名匹配的书签（⚠️ 写操作，建议先加 --dry-run）
    Rename {
        /// 关键词（匹配书签名或 URL）
        keyword: Vec<String>,
        /// 新名称
        #[arg(long)]
        name: String,
        /// 预览变更，不实际修改
        #[arg(long)]
        dry_run: bool,
    },
    /// 统计书签数量分布（按文件夹 / 域名）
    Stats {
        /// 显示 Top N 域名（默认 20）
        #[arg(long, default_value = "20")]
        top: usize,
    },
    /// 对指定文件夹内书签按名称排序（⚠️ 写操作，建议先加 --dry-run）
    Sort {
        /// 文件夹路径，如 "书签栏/工具"
        folder: Vec<String>,
        /// 预览变更，不实际修改
        #[arg(long)]
        dry_run: bool,
    },
    /// 新建文件夹（⚠️ 写操作，建议先加 --dry-run）
    Mkdir {
        /// 完整路径，如 "书签栏/云服务器/新分类"
        path: Vec<String>,
        /// 预览变更，不实际修改
        #[arg(long)]
        dry_run: bool,
    },
    /// 移动文件夹到指定位置（⚠️ 写操作，建议先加 --dry-run）
    Mvdir {
        /// 源文件夹路径，如 "书签栏/云服务器/旧分类"
        source: Vec<String>,
        /// 目标父文件夹路径，如 "书签栏/技术开发"
        #[arg(long)]
        to: String,
        /// 预览变更，不实际修改
        #[arg(long)]
        dry_run: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let (_, path) = finder::find(cli.profile.as_deref())?;
    eprintln!("[chrome  {}]\n", path.display());

    match cli.command.unwrap_or(Commands::Structure) {
        Commands::Structure   => cmd::structure::run(&path)?,
        Commands::Dupes       => cmd::dupes::run(&path)?,
        Commands::Analyze     => cmd::analyze::run(&path)?,
        Commands::Search { query } => cmd::search::run(&path, &query.join(" "))?,
        Commands::Deadlinks { concurrency, timeout } => {
            cmd::deadlinks::run(&path, concurrency, timeout).await?
        }
        Commands::Delete { keyword, dry_run } => {
            cmd::delete::run(&path, &keyword.join(" "), dry_run)?
        }
        Commands::Mv { keyword, to, dry_run } => {
            cmd::mv::run(&path, &keyword.join(" "), &to, dry_run)?
        }
        Commands::Rename { keyword, name, dry_run } => {
            cmd::rename::run(&path, &keyword.join(" "), &name, dry_run)?
        }
        Commands::Stats { top } => cmd::stats::run(&path, top)?,
        Commands::Sort { folder, dry_run } => {
            cmd::sort::run(&path, &folder.join(" "), dry_run)?
        }
        Commands::Mkdir { path: folder_path, dry_run } => {
            cmd::mkdir::run(&path, &folder_path.join(" "), dry_run)?
        }
        Commands::Mvdir { source, to, dry_run } => {
            cmd::mvdir::run(&path, &source.join(" "), &to, dry_run)?
        }
    }

    Ok(())
}
