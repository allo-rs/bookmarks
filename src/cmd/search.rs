use std::path::Path;
use anyhow::Result;
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;

use crate::bookmark;

pub fn run(path: &Path, query: &str) -> Result<()> {
    let file = bookmark::load(path)?;
    let all = bookmark::get_all_bookmarks(&file);

    let matcher = SkimMatcherV2::default();
    let terms = tokenize(query);

    // 所有词都命中才算匹配（AND），分数求和排序
    let mut scored: Vec<(i64, _)> = all.iter()
        .filter_map(|bm| {
            let mut total = 0i64;
            for term in &terms {
                let name_score = matcher.fuzzy_match(&bm.name, term);
                let url_score  = matcher.fuzzy_match(&bm.url,  term).map(|s| s * 7 / 10);
                // 任一维度命中即可，取最高分；若该词完全没命中则整条丢弃
                let best = [name_score, url_score].into_iter().flatten().max();
                total += best?;   // ? 使 None 时 filter_map 返回 None
            }
            Some((total, bm))
        })
        .collect();

    scored.sort_by_key(|(score, _)| std::cmp::Reverse(*score));

    let total = scored.len();
    println!("搜索 \"{query}\"，匹配 {total} 条（按相关度排序）:\n");
    for (_, bm) in scored.iter().take(30) {
        println!("📁 {}", bm.path);
        println!("   {}", trunc(&bm.name, 70));
        println!("   {}", trunc(&bm.url, 80));
        println!();
    }
    if total > 30 {
        println!("... 还有 {} 条，请缩小搜索范围", total - 30);
    }

    Ok(())
}

/// 先按空格分词，再在 CJK/ASCII 边界切分
/// "k8s部署"  → ["k8s", "部署"]
/// "react hook" → ["react", "hook"]
fn tokenize(query: &str) -> Vec<String> {
    let mut tokens = vec![];
    for word in query.split_whitespace() {
        split_cjk(word, &mut tokens);
    }
    tokens
}

fn split_cjk(s: &str, out: &mut Vec<String>) {
    let mut buf = String::new();
    let mut last_cjk: Option<bool> = None;

    for ch in s.chars() {
        let is_cjk = matches!(ch,
            '\u{4e00}'..='\u{9fff}'   // CJK 统一汉字
            | '\u{3400}'..='\u{4dbf}' // CJK 扩展 A
            | '\u{f900}'..='\u{faff}' // CJK 兼容汉字
            | '\u{3000}'..='\u{303f}' // CJK 符号和标点
        );
        if last_cjk.is_some_and(|prev| prev != is_cjk) {
            if !buf.is_empty() {
                out.push(buf.clone());
                buf.clear();
            }
        }
        buf.push(ch);
        last_cjk = Some(is_cjk);
    }
    if !buf.is_empty() {
        out.push(buf);
    }
}

fn trunc(s: &str, max: usize) -> String {
    let mut chars = s.chars();
    let out: String = chars.by_ref().take(max).collect();
    if chars.next().is_some() { format!("{out}…") } else { out }
}
