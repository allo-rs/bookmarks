use std::collections::HashMap;
use std::path::Path;
use anyhow::Result;
use crate::bookmark;

pub fn run(path: &Path) -> Result<()> {
    let file = bookmark::load(path)?;
    let all = bookmark::get_all_bookmarks(&file);
    let stats = bookmark::get_folder_stats(&file);

    println!("总书签数: {}，总文件夹数: {}\n", all.len(), stats.len());

    let overloaded: Vec<_> = stats.iter().filter(|f| f.url_count > 30).collect();
    if !overloaded.is_empty() {
        println!("=== 书签过多（>30）建议细分 ===");
        let mut sorted = overloaded;
        sorted.sort_by_key(|f| std::cmp::Reverse(f.url_count));
        for f in sorted {
            println!("  {:4} 个  {}", f.url_count, f.path);
        }
        println!();
    }

    let has_both: Vec<_> = stats.iter().filter(|f| f.url_count > 0 && f.folder_count > 0).collect();
    if !has_both.is_empty() {
        println!("=== 父文件夹有散落书签（建议归入子文件夹）===");
        let mut sorted = has_both;
        sorted.sort_by_key(|f| std::cmp::Reverse(f.url_count));
        for f in sorted {
            println!("  {:3} 个散落  📁 {}", f.url_count, f.path);
        }
        println!();
    }

    let sparse: Vec<_> = stats.iter().filter(|f| f.url_count > 0 && f.url_count <= 3 && f.folder_count == 0).collect();
    if !sparse.is_empty() {
        println!("=== 书签过少（≤3）可考虑合并 ===");
        let mut sorted = sparse;
        sorted.sort_by_key(|f| f.url_count);
        for f in sorted {
            println!("  {:3} 个  {}", f.url_count, f.path);
        }
        println!();
    }

    let mut by_domain: HashMap<String, usize> = HashMap::new();
    for bm in &all {
        if let Some(host) = extract_host(&bm.url) {
            *by_domain.entry(host).or_default() += 1;
        }
    }
    let mut domains: Vec<_> = by_domain.into_iter().collect();
    domains.sort_by_key(|(_, c)| std::cmp::Reverse(*c));

    println!("=== 域名分布 Top 15 ===");
    for (domain, count) in domains.iter().take(15) {
        println!("  {count:4}  {domain}");
    }

    Ok(())
}

fn extract_host(url: &str) -> Option<String> {
    let after_scheme = url.split("://").nth(1)?;
    let host = after_scheme.split('/').next()?;
    Some(host.to_string())
}
