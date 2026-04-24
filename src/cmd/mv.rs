use std::collections::HashSet;
use std::path::Path;
use anyhow::{bail, Result};

use crate::write::{self, trunc};

pub fn run(path: &Path, keyword: &str, target_folder: &str, dry_run: bool) -> Result<()> {
    let data = write::load_raw(path)?;

    // 验证目标文件夹存在
    let folder_id = write::find_folder_id(&data, target_folder)
        .ok_or_else(|| anyhow::anyhow!("目标文件夹不存在：「{target_folder}」"))?;

    let matches = write::find_matching(&data, keyword);
    if matches.is_empty() {
        println!("未找到匹配「{keyword}」的书签。");
        return Ok(());
    }

    println!("将移动以下 {} 条书签 → 📁 {target_folder}\n", matches.len());
    for m in &matches {
        println!("  {}", trunc(&m.name, 60));
        println!("  {}", trunc(&m.url, 80));
        println!("  从：📁 {}", m.folder);
        println!();
    }

    if dry_run {
        println!("[DRY RUN] 未做任何修改。去掉 --dry-run 执行变更。");
        return Ok(());
    }

    let backup = write::backup(path)?;
    println!("备份已保存：{}", backup.display());

    let ids: HashSet<String> = matches.iter().map(|m| m.id.clone()).collect();
    let (new_data, extracted) = write::extract_from_roots(&data, &ids);

    if extracted.is_empty() {
        bail!("提取节点失败，操作取消");
    }

    let new_data = write::insert_into_roots(&new_data, &folder_id, &extracted)?;
    write::save(path, &new_data)?;

    println!("✓ 已将 {} 条书签移动到「{target_folder}」", extracted.len());
    Ok(())
}
