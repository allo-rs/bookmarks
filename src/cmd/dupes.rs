use std::collections::HashMap;
use std::path::Path;
use anyhow::Result;
use crate::bookmark;

pub fn run(path: &Path) -> Result<()> {
    let file = bookmark::load(path)?;
    let all = bookmark::get_all_bookmarks(&file);

    // URL 重复
    let mut by_url: HashMap<String, Vec<_>> = HashMap::new();
    for bm in &all {
        by_url.entry(bm.url.trim_end_matches('/').to_string()).or_default().push(bm);
    }
    let mut url_dupes: Vec<_> = by_url.into_iter().filter(|(_, v)| v.len() > 1).collect();
    url_dupes.sort_by_key(|(_, v)| std::cmp::Reverse(v.len()));

    println!("=== URL 完全重复 ===");
    if url_dupes.is_empty() {
        println!("无 URL 完全重复的书签\n");
    } else {
        for (url, bms) in &url_dupes {
            println!("[{}次] {}", bms.len(), truncate(&bms[0].name, 60));
            println!("  URL: {}", truncate(url, 80));
            for bm in bms.iter() {
                println!("  📁 {}", bm.path);
            }
            println!();
        }
    }

    // 名称重复（URL 不同）
    let mut by_name: HashMap<String, Vec<_>> = HashMap::new();
    for bm in &all {
        let n = bm.name.trim().to_string();
        if !n.is_empty() {
            by_name.entry(n).or_default().push(bm);
        }
    }
    let mut name_dupes: Vec<_> = by_name.into_iter().filter(|(_, v)| v.len() > 1).collect();
    name_dupes.sort_by_key(|(_, v)| std::cmp::Reverse(v.len()));

    println!("=== 名称重复（URL 不同）===");
    if name_dupes.is_empty() {
        println!("无名称重复的书签\n");
    } else {
        for (name, bms) in name_dupes.iter().take(20) {
            println!("[{}次] {}", bms.len(), truncate(name, 60));
            for bm in bms.iter() {
                println!("  📁 {}", bm.path);
                println!("     {}", truncate(&bm.url, 70));
            }
            println!();
        }
    }

    Ok(())
}

fn truncate(s: &str, max: usize) -> String {
    let mut chars = s.chars();
    let out: String = chars.by_ref().take(max).collect();
    if chars.next().is_some() { format!("{out}…") } else { out }
}
