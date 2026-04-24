use serde::Deserialize;
use std::path::Path;
use anyhow::Result;

#[derive(Deserialize)]
pub struct BookmarkFile {
    pub roots: Roots,
}

#[derive(Deserialize)]
pub struct Roots {
    pub bookmark_bar: Node,
    pub other: Node,
    pub synced: Option<Node>,
}

#[derive(Deserialize, Clone)]
pub struct Node {
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub kind: Option<String>,
    pub url: Option<String>,
    pub children: Option<Vec<Node>>,
}

#[derive(Debug, Clone)]
pub struct Bookmark {
    pub name: String,
    pub url: String,
    pub path: String,
}

pub struct FolderStat {
    pub path: String,
    pub url_count: usize,
    pub folder_count: usize,
}

pub fn load(path: &Path) -> Result<BookmarkFile> {
    let data = std::fs::read_to_string(path)?;
    Ok(serde_json::from_str(&data)?)
}

pub fn get_all_bookmarks(file: &BookmarkFile) -> Vec<Bookmark> {
    let mut all = vec![];
    for node in roots_iter(file) {
        extract_all(node, "", &mut all);
    }
    all
}

pub fn get_folder_stats(file: &BookmarkFile) -> Vec<FolderStat> {
    let mut stats = vec![];
    for node in roots_iter(file) {
        collect_folder_stats(node, "", &mut stats);
    }
    stats
}

fn roots_iter(file: &BookmarkFile) -> impl Iterator<Item = &Node> {
    [&file.roots.bookmark_bar, &file.roots.other]
        .into_iter()
        .chain(file.roots.synced.as_ref())
}

pub fn extract_all(node: &Node, path: &str, out: &mut Vec<Bookmark>) {
    let name = node.name.as_deref().unwrap_or("");
    if node.kind.as_deref() == Some("url") {
        if let Some(url) = &node.url {
            out.push(Bookmark { name: name.to_string(), url: url.clone(), path: path.to_string() });
        }
    } else if let Some(children) = &node.children {
        let new_path = join_path(path, name);
        for child in children {
            extract_all(child, &new_path, out);
        }
    }
}

fn collect_folder_stats(node: &Node, path: &str, stats: &mut Vec<FolderStat>) {
    let name = node.name.as_deref().unwrap_or("");
    let Some(children) = &node.children else { return };
    let new_path = join_path(path, name);

    let url_count = children.iter().filter(|c| c.kind.as_deref() == Some("url")).count();
    let folders: Vec<_> = children.iter().filter(|c| c.children.is_some()).collect();

    if !name.is_empty() {
        stats.push(FolderStat { path: new_path.clone(), url_count, folder_count: folders.len() });
    }
    for f in folders {
        collect_folder_stats(f, &new_path, stats);
    }
}

fn join_path(parent: &str, name: &str) -> String {
    if name.is_empty() { parent.to_string() }
    else if parent.is_empty() { name.to_string() }
    else { format!("{parent}/{name}") }
}
