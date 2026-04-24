use std::collections::HashSet;
use std::path::Path;
use anyhow::Result;

use crate::write::{self, trunc};

pub fn run(path: &Path, keyword: &str, new_name: &str, dry_run: bool) -> Result<()> {
    let data = write::load_raw(path)?;
    let matches = write::find_matching(&data, keyword);

    if matches.is_empty() {
        println!("未找到匹配「{keyword}」的书签。");
        return Ok(());
    }

    println!("将重命名以下 {} 条书签 → 「{new_name}」\n", matches.len());
    for m in &matches {
        println!("  {} → {new_name}", trunc(&m.name, 60));
        println!("  {}", trunc(&m.url, 80));
        println!("  📁 {}", m.folder);
        println!();
    }

    if dry_run {
        println!("[DRY RUN] 未做任何修改。去掉 --dry-run 执行变更。");
        return Ok(());
    }

    let backup = write::backup(path)?;
    println!("备份已保存：{}", backup.display());

    let ids: HashSet<String> = matches.iter().map(|m| m.id.clone()).collect();
    let new_data = write::rename_in_roots(&data, &ids, new_name);
    write::save(path, &new_data)?;

    println!("✓ 已将 {} 条书签重命名为「{new_name}」", matches.len());
    Ok(())
}
