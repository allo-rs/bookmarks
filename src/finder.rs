use std::path::PathBuf;
use anyhow::{bail, Result};

pub fn find(profile: Option<&str>) -> Result<(String, PathBuf)> {
    let home: PathBuf = std::env::var("HOME").unwrap_or_default().into();
    let base = home.join("Library/Application Support/Google/Chrome");

    let mut paths = bookmark_paths(&base);
    paths.sort();

    if paths.is_empty() {
        bail!("未找到 Chrome 书签文件，请确认 Chrome 已安装");
    }

    if let Some(n) = profile {
        let filtered: Vec<_> = paths
            .iter()
            .filter(|p| p.to_string_lossy().contains(&format!("Profile {n}")))
            .cloned()
            .collect();
        if !filtered.is_empty() {
            return Ok(("chrome".to_string(), filtered.into_iter().next().unwrap()));
        }
    }

    Ok(("chrome".to_string(), paths.into_iter().next().unwrap()))
}

fn bookmark_paths(base: &std::path::Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(base) else { return vec![] };
    entries
        .filter_map(|e| e.ok())
        .filter(|e| {
            let name = e.file_name();
            let n = name.to_string_lossy();
            n != "Guest Profile" && n != "System Profile"
        })
        .map(|e| e.path().join("Bookmarks"))
        .filter(|p| p.exists())
        .collect()
}
