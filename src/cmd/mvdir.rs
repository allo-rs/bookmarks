use std::collections::HashSet;
use std::path::Path;
use anyhow::Result;
use crate::write;

pub fn run(path: &Path, source_path: &str, target_path: &str, dry_run: bool) -> Result<()> {
    let data = write::load_raw(path)?;

    let source_id = write::find_folder_id(&data, source_path)
        .ok_or_else(|| anyhow::anyhow!("源文件夹不存在：「{source_path}」"))?;

    let target_id = write::find_folder_id(&data, target_path)
        .ok_or_else(|| anyhow::anyhow!("目标文件夹不存在：「{target_path}」"))?;

    if target_id == source_id || target_path.starts_with(&format!("{source_path}/")) {
        anyhow::bail!("不能将文件夹移动到自身或其子文件夹内");
    }

    let folder_name = source_path.split('/').last().unwrap_or(source_path);
    let bm_count = write::count_folder_bookmarks(&data, source_path);

    println!("将移动文件夹（含 {bm_count} 个书签）：");
    println!("  📁 {source_path}");
    println!("  → 📁 {target_path}/{folder_name}");

    if dry_run {
        println!("\n[DRY RUN] 未做任何修改。去掉 --dry-run 执行变更。");
        return Ok(());
    }

    let ids: HashSet<String> = HashSet::from([source_id]);
    let (new_data, extracted) = write::extract_from_roots(&data, &ids);

    if extracted.is_empty() {
        anyhow::bail!("提取文件夹失败，操作取消");
    }

    let backup = write::backup(path)?;
    println!("\n备份已保存：{}", backup.display());

    let new_data = write::insert_into_roots(&new_data, &target_id, &extracted)?;
    write::save(path, &new_data)?;

    println!("✓ 已将「{folder_name}」移动到「{target_path}」");
    Ok(())
}
