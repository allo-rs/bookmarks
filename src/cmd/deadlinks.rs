use std::collections::HashMap;
use std::error::Error as StdError;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use tokio::sync::Semaphore;

use crate::bookmark::{self, Bookmark};

enum Status {
    Ok,
    Moved { final_url: String },
    HttpError(u16),
    Timeout,
    ConnectFailed(String),
}

pub async fn run(path: &Path, concurrency: usize, timeout_secs: u64) -> Result<()> {
    let file = bookmark::load(path)?;
    let all = bookmark::get_all_bookmarks(&file);

    // 按 URL 分组，去重后每组只发一次请求
    let mut groups: HashMap<String, Vec<Bookmark>> = HashMap::new();
    for bm in all {
        if bm.url.starts_with("http://") || bm.url.starts_with("https://") {
            groups.entry(bm.url.clone()).or_default().push(bm);
        }
    }

    let unique = groups.len();
    let total: usize = groups.values().map(|v| v.len()).sum();
    println!("共 {total} 个书签，{unique} 个唯一 URL（并发 {concurrency}，超时 {timeout_secs}s）\n");

    let client = Arc::new(
        Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36")
            .redirect(reqwest::redirect::Policy::limited(5))
            .build()?,
    );
    let sem = Arc::new(Semaphore::new(concurrency));
    let pb = Arc::new(ProgressBar::new(unique as u64).with_style(
        ProgressStyle::with_template(
            "  {spinner:.green} [{bar:40.cyan/blue}] {pos}/{len}  {elapsed_precise}  ETA {eta}",
        )?
        .progress_chars("█▉▊▋▌▍▎▏  "),
    ));

    let mut handles = Vec::with_capacity(unique);
    for (url, bms) in groups {
        let client = Arc::clone(&client);
        let sem = Arc::clone(&sem);
        let pb = Arc::clone(&pb);
        handles.push(tokio::spawn(async move {
            let _permit = sem.acquire().await.expect("semaphore closed");
            let status = check(&client, &url).await;
            pb.inc(1);
            (status, bms)
        }));
    }

    let mut results: Vec<(Status, Vec<Bookmark>)> = Vec::with_capacity(unique);
    for h in handles {
        results.push(h.await?);
    }
    pb.finish_and_clear();

    report(&results, total);
    Ok(())
}

async fn check(client: &Client, url: &str) -> Status {
    match client.get(url).send().await {
        Ok(resp) => {
            let code = resp.status().as_u16();
            let final_url = resp.url().as_str().to_string();
            if !(200..300).contains(&code) {
                Status::HttpError(code)
            } else if norm(&final_url) != norm(url) {
                Status::Moved { final_url }
            } else {
                Status::Ok
            }
        }
        Err(e) if e.is_timeout() => Status::Timeout,
        Err(e) => Status::ConnectFailed(
            e.source().map(|s: &dyn StdError| s.to_string()).unwrap_or_else(|| e.to_string()),
        ),
    }
}

// 归一化用于比较：去掉末尾斜线，小写化协议+主机部分
fn norm(url: &str) -> String {
    url.trim_end_matches('/').to_lowercase()
}

fn report(results: &[(Status, Vec<Bookmark>)], total: usize) {
    let mut http_errors: Vec<_> = results.iter().filter(|(s, _)| matches!(s, Status::HttpError(_))).collect();
    let timeouts: Vec<_>       = results.iter().filter(|(s, _)| matches!(s, Status::Timeout)).collect();
    let failed: Vec<_>         = results.iter().filter(|(s, _)| matches!(s, Status::ConnectFailed(_))).collect();
    let moved: Vec<_>          = results.iter().filter(|(s, _)| matches!(s, Status::Moved { .. })).collect();
    let ok                     = results.iter().filter(|(s, _)| matches!(s, Status::Ok)).count();

    http_errors.sort_by_key(|(s, _)| if let Status::HttpError(c) = s { *c } else { 0 });

    if !http_errors.is_empty() {
        println!("=== HTTP 错误 ({}) ===", http_errors.len());
        for (status, bms) in &http_errors {
            if let Status::HttpError(code) = status {
                for bm in bms.iter() {
                    println!("  [{code}]  {}", trunc(&bm.name, 60));
                    println!("         {}", trunc(&bm.url, 90));
                    println!("         📁 {}", bm.path);
                }
            }
        }
        println!();
    }

    if !timeouts.is_empty() {
        println!("=== 超时 ({}) ===", timeouts.len());
        for (_, bms) in &timeouts {
            for bm in bms.iter() {
                println!("  {}  📁 {}", trunc(&bm.url, 80), bm.path);
            }
        }
        println!();
    }

    if !failed.is_empty() {
        println!("=== 连接失败 ({}) ===", failed.len());
        for (status, bms) in &failed {
            if let Status::ConnectFailed(msg) = status {
                for bm in bms.iter() {
                    println!("  {}  📁 {}", trunc(&bm.url, 80), bm.path);
                }
                println!("  原因: {}", trunc(msg, 80));
            }
        }
        println!();
    }

    if !moved.is_empty() {
        println!("=== 已重定向（仍可访问，供参考）({}) ===", moved.len());
        for (status, bms) in moved.iter().take(20) {
            if let Status::Moved { final_url } = status {
                for bm in bms.iter() {
                    println!("  {}  →  {}", trunc(&bm.url, 55), trunc(final_url, 55));
                    println!("  📁 {}", bm.path);
                }
            }
        }
        if moved.len() > 20 {
            println!("  ... 还有 {} 条", moved.len() - 20);
        }
        println!();
    }

    let bad = http_errors.len() + timeouts.len() + failed.len();
    let bad_pct = if total > 0 { bad as f64 / total as f64 * 100.0 } else { 0.0 };
    println!("=== 汇总 ===");
    println!("总计 {total}  ✅ {ok}  🔀 重定向 {}  ❌ 异常 {bad} ({bad_pct:.1}%)", moved.len());
    println!("       HTTP错误 {}  超时 {}  失败 {}", http_errors.len(), timeouts.len(), failed.len());
}

fn trunc(s: &str, max: usize) -> String {
    let mut chars = s.chars();
    let out: String = chars.by_ref().take(max).collect();
    if chars.next().is_some() { format!("{out}…") } else { out }
}
