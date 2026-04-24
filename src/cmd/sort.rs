use std::path::Path;
use anyhow::Result;

use crate::write;

pub fn run(path: &Path, folder_path: &str, dry_run: bool) -> Result<()> {
    let data = write::load_raw(path)?;

    let folder_id = write::find_folder_id(&data, folder_path)
        .ok_or_else(|| anyhow::anyhow!("文件夹不存在：「{folder_path}」"))?;

    let (sorted_data, count) = write::sort_folder_in_roots(&data, &folder_id);

    println!("将对「{folder_path}」内 {count} 个子项按名称排序（文件夹优先）");

    if dry_run {
        println!("[DRY RUN] 未做任何修改。去掉 --dry-run 执行变更。");
        return Ok(());
    }

    let backup = write::backup(path)?;
    println!("备份已保存：{}", backup.display());

    write::save(path, &sorted_data)?;
    println!("✓ 已排序「{folder_path}」");
    Ok(())
}
