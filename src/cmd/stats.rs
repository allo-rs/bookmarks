use std::collections::HashMap;
use std::path::Path;
use anyhow::Result;

use crate::bookmark;

pub fn run(path: &Path, top: usize) -> Result<()> {
    let file = bookmark::load(path)?;
    let all = bookmark::get_all_bookmarks(&file);
    let total = all.len();

    // 按顶级文件夹统计
    let mut by_root: HashMap<&str, usize> = HashMap::new();
    for bm in &all {
        let root = bm.path.split('/').next().unwrap_or("(未知)");
        *by_root.entry(root).or_default() += 1;
    }

    // 按域名统计
    let mut by_domain: HashMap<String, usize> = HashMap::new();
    for bm in &all {
        let domain = extract_domain(&bm.url);
        *by_domain.entry(domain).or_default() += 1;
    }

    let mut domain_list: Vec<_> = by_domain.iter().collect();
    domain_list.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));

    println!("=== 书签总览 ===");
    println!("总计 {total} 个书签\n");

    println!("── 按顶级文件夹 ──");
    let mut root_list: Vec<_> = by_root.iter().collect();
    root_list.sort_by(|a, b| b.1.cmp(a.1));
    for (name, count) in &root_list {
        let pct = *count * 100 / total;
        println!("  {name:<14}  {count:>5}  ({pct}%)");
    }

    println!("\n── 按域名 Top {top} ──");
    for (domain, count) in domain_list.iter().take(top) {
        println!("  {domain:<40}  {count:>5}");
    }

    let unique_domains = by_domain.len();
    println!("\n共 {unique_domains} 个不同域名");

    Ok(())
}

fn extract_domain(url: &str) -> String {
    // 去掉协议头，取 host 部分
    let s = url
        .trim_start_matches("https://")
        .trim_start_matches("http://");
    let host = s.split('/').next().unwrap_or(s);
    // 去掉端口号
    let host = host.split(':').next().unwrap_or(host);
    // 去掉 www. 前缀
    host.trim_start_matches("www.").to_lowercase()
}
