use clap::{Parser, Subcommand};
use anyhow::Result;

mod bookmark;
mod cmd;
mod finder;

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
    /// 搜索书签
    Search { query: Vec<String> },
    /// 并发检测死链
    Deadlinks {
        /// 并发数（默认 100）
        #[arg(long, default_value = "100")]
        concurrency: usize,
        /// 超时秒数（默认 8）
        #[arg(long, default_value = "8")]
        timeout: u64,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let (_, path) = finder::find(cli.profile.as_deref())?;
    eprintln!("[chrome  {}]\n", path.display());

    match cli.command.unwrap_or(Commands::Structure) {
        Commands::Structure => cmd::structure::run(&path)?,
        Commands::Dupes => cmd::dupes::run(&path)?,
        Commands::Analyze => cmd::analyze::run(&path)?,
        Commands::Search { query } => cmd::search::run(&path, &query.join(" "))?,
        Commands::Deadlinks { concurrency, timeout } => {
            cmd::deadlinks::run(&path, concurrency, timeout).await?
        }
    }

    Ok(())
}
