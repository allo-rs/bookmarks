use std::collections::HashSet;
use std::path::Path;
use anyhow::Result;

use crate::write::{self, trunc};

pub fn run(path: &Path, keyword: &str, dry_run: bool) -> Result<()> {
    let data = write::load_raw(path)?;
    let matches = write::find_matching(&data, keyword);

    if matches.is_empty() {
        println!("未找到匹配「{keyword}」的书签。");
        return Ok(());
    }

    println!("将删除以下 {} 条书签:\n", matches.len());
    for m in &matches {
        println!("  - {}", trunc(&m.name, 60));
        println!("    {}", trunc(&m.url, 80));
        println!("    📁 {}", m.folder);
    }

    if dry_run {
        println!("\n[DRY RUN] 未做任何修改。去掉 --dry-run 执行变更。");
        return Ok(());
    }

    let backup = write::backup(path)?;
    println!("\n备份已保存：{}", backup.display());

    let ids: HashSet<String> = matches.iter().map(|m| m.id.clone()).collect();
    let new_data = write::map_roots(&data, |root| write::remove_ids(root, &ids));
    write::save(path, &new_data)?;

    println!("✓ 已删除 {} 条书签", matches.len());
    Ok(())
}
