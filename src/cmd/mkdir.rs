use std::path::Path;
use anyhow::Result;
use crate::write;

pub fn run(path: &Path, folder_path: &str, dry_run: bool) -> Result<()> {
    let data = write::load_raw(path)?;

    let idx = folder_path.rfind('/')
        .ok_or_else(|| anyhow::anyhow!("路径须含父文件夹，如 '书签栏/云服务器/新分类'"))?;
    let parent_path = &folder_path[..idx];
    let folder_name = &folder_path[idx + 1..];

    write::find_folder_id(&data, parent_path)
        .ok_or_else(|| anyhow::anyhow!("父文件夹不存在：「{parent_path}」"))?;

    if write::find_folder_id(&data, folder_path).is_some() {
        anyhow::bail!("文件夹已存在：「{folder_path}」");
    }

    println!("将在 📁 {parent_path} 下新建：");
    println!("  📁 {folder_name}");

    if dry_run {
        println!("\n[DRY RUN] 未做任何修改。去掉 --dry-run 执行变更。");
        return Ok(());
    }

    let backup = write::backup(path)?;
    println!("\n备份已保存：{}", backup.display());

    let new_data = write::create_folder_in_roots(&data, folder_path)?;
    write::save(path, &new_data)?;

    println!("✓ 已新建「{folder_path}」");
    Ok(())
}
