use std::collections::HashSet;
use std::path::{Path, PathBuf};
use anyhow::{bail, Result};
use serde_json::Value;

#[derive(Debug)]
pub struct BmMatch {
    pub id: String,
    pub name: String,
    pub url: String,
    pub folder: String,
}

pub fn load_raw(path: &Path) -> Result<Value> {
    let s = std::fs::read_to_string(path)?;
    Ok(serde_json::from_str(&s)?)
}

pub fn save(path: &Path, data: &Value) -> Result<()> {
    let json = serde_json::to_string_pretty(data)?;
    std::fs::write(path, json)?;
    Ok(())
}

pub fn backup(path: &Path) -> Result<PathBuf> {
    use std::time::{SystemTime, UNIX_EPOCH};
    let ts = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis();
    let backup = path.with_file_name(format!("Bookmarks.bm-{ts}"));
    std::fs::copy(path, &backup)?;
    cleanup_old_backups(path, 3);
    Ok(backup)
}

fn cleanup_old_backups(path: &Path, keep: usize) {
    let dir = match path.parent() { Some(d) => d, None => return };
    let mut backups: Vec<_> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| {
            e.file_name().to_string_lossy().starts_with("Bookmarks.bm-")
        })
        .collect();
    if backups.len() <= keep { return; }
    backups.sort_by_key(|e| e.file_name());
    for entry in &backups[..backups.len() - keep] {
        let _ = std::fs::remove_file(entry.path());
    }
}

/// 按关键词搜索书签（名称或 URL 包含即命中）
pub fn find_matching(data: &Value, keyword: &str) -> Vec<BmMatch> {
    let mut results = vec![];
    let q = keyword.to_lowercase();
    if let Some(roots) = data["roots"].as_object() {
        for root in roots.values() {
            collect_matching(root, &q, "", &mut results);
        }
    }
    results
}

fn collect_matching(node: &Value, q: &str, path: &str, out: &mut Vec<BmMatch>) {
    let name = node["name"].as_str().unwrap_or("");
    if node["type"].as_str() == Some("url") {
        let url = node["url"].as_str().unwrap_or("");
        if name.to_lowercase().contains(q) || url.to_lowercase().contains(q) {
            if let Some(id) = node["id"].as_str() {
                out.push(BmMatch {
                    id: id.to_string(),
                    name: name.to_string(),
                    url: url.to_string(),
                    folder: path.to_string(),
                });
            }
        }
    } else if let Some(children) = node["children"].as_array() {
        let new_path = join_path(path, name);
        for child in children {
            collect_matching(child, q, &new_path, out);
        }
    }
}

/// 从树中删除指定 ID 的节点（clone-based，保留所有原始字段）
pub fn remove_ids(node: &Value, ids: &HashSet<String>) -> Value {
    let mut new_node = node.clone();
    if let Some(children) = node["children"].as_array() {
        let new_children: Vec<Value> = children
            .iter()
            .filter(|c| !ids.contains(c["id"].as_str().unwrap_or("")))
            .map(|c| remove_ids(c, ids))
            .collect();
        new_node["children"] = Value::Array(new_children);
    }
    new_node
}

/// 从树中提取指定 ID 的节点，返回 (新树, 被提取的节点列表)
pub fn extract_ids(node: &Value, ids: &HashSet<String>) -> (Value, Vec<Value>) {
    let mut new_node = node.clone();
    let mut extracted = vec![];
    if let Some(children) = node["children"].as_array() {
        let mut new_children = vec![];
        for child in children {
            if ids.contains(child["id"].as_str().unwrap_or("")) {
                extracted.push(child.clone());
            } else {
                let (new_child, sub) = extract_ids(child, ids);
                new_children.push(new_child);
                extracted.extend(sub);
            }
        }
        new_node["children"] = Value::Array(new_children);
    }
    (new_node, extracted)
}

/// 将节点列表插入指定 folder_id 的 children 末尾，返回 (新树, 是否找到目标)
pub fn insert_into(node: &Value, folder_id: &str, items: &[Value]) -> (Value, bool) {
    let mut new_node = node.clone();
    let id = node["id"].as_str().unwrap_or("");

    if !folder_id.is_empty() && id == folder_id {
        if let Some(children) = new_node["children"].as_array_mut() {
            children.extend_from_slice(items);
            return (new_node, true);
        }
    }

    if let Some(children) = node["children"].as_array() {
        let mut new_children = vec![];
        let mut found = false;
        for child in children {
            if found {
                new_children.push(child.clone());
            } else {
                let (new_child, child_found) = insert_into(child, folder_id, items);
                new_children.push(new_child);
                found = child_found;
            }
        }
        new_node["children"] = Value::Array(new_children);
        return (new_node, found);
    }

    (new_node, false)
}

/// 按路径查找文件夹 ID（如 "书签栏/云服务器/网络监控"）
pub fn find_folder_id(data: &Value, folder_path: &str) -> Option<String> {
    let parts: Vec<&str> = folder_path.split('/').collect();
    let roots = data["roots"].as_object()?;

    let root_node = match parts[0] {
        "书签栏" => roots.get("bookmark_bar")?,
        "其他书签" => roots.get("other")?,
        "移动设备书签" => roots.get("synced")?,
        _ => return None,
    };

    let mut current = root_node;
    for part in parts.iter().skip(1) {
        let children = current["children"].as_array()?;
        current = children.iter().find(|c| {
            c["name"].as_str() == Some(part) && c["type"].as_str() != Some("url")
        })?;
    }
    current["id"].as_str().filter(|s| !s.is_empty()).map(|s| s.to_string())
}

/// 对 roots 对象的每个 root 应用变换 f，返回新的 roots Value
pub fn map_roots(data: &Value, mut f: impl FnMut(&Value) -> Value) -> Value {
    let mut new_data = data.clone();
    if let Some(roots) = new_data["roots"].as_object_mut() {
        for root in roots.values_mut() {
            *root = f(root);
        }
    }
    new_data
}

/// 对 roots 对象的每个 root 应用 extract_ids，汇总所有被提取的节点
pub fn extract_from_roots(data: &Value, ids: &HashSet<String>) -> (Value, Vec<Value>) {
    let mut new_data = data.clone();
    let mut all_extracted = vec![];
    if let Some(roots) = new_data["roots"].as_object_mut() {
        for root in roots.values_mut() {
            let (new_root, extracted) = extract_ids(root, ids);
            *root = new_root;
            all_extracted.extend(extracted);
        }
    }
    (new_data, all_extracted)
}

/// 对 roots 的每个 root 插入节点到指定 folder_id
pub fn insert_into_roots(data: &Value, folder_id: &str, items: &[Value]) -> Result<Value> {
    let mut new_data = data.clone();
    if let Some(roots) = new_data["roots"].as_object_mut() {
        for root in roots.values_mut() {
            let (new_root, found) = insert_into(root, folder_id, items);
            *root = new_root;
            if found {
                return Ok(new_data);
            }
        }
    }
    bail!("目标文件夹未找到");
}

/// 将指定 ID 的书签重命名
pub fn rename_in_roots(data: &Value, ids: &HashSet<String>, new_name: &str) -> Value {
    map_roots(data, |root| rename_ids(root, ids, new_name))
}

fn rename_ids(node: &Value, ids: &HashSet<String>, new_name: &str) -> Value {
    let mut new_node = node.clone();
    if let Some(id) = node["id"].as_str() {
        if ids.contains(id) {
            new_node["name"] = Value::String(new_name.to_string());
            return new_node;
        }
    }
    if let Some(children) = node["children"].as_array() {
        let new_children: Vec<Value> = children.iter().map(|c| rename_ids(c, ids, new_name)).collect();
        new_node["children"] = Value::Array(new_children);
    }
    new_node
}

/// 对指定 folder_id 的 children 排序（文件夹优先，各组内按名称字典序）
/// 返回 (新数据, 排序的子项数量)
pub fn sort_folder_in_roots(data: &Value, folder_id: &str) -> (Value, usize) {
    let mut new_data = data.clone();
    let mut count = 0;
    if let Some(roots) = new_data["roots"].as_object_mut() {
        for root in roots.values_mut() {
            if let Some(c) = sort_in_tree(root, folder_id) {
                count = c;
                break;
            }
        }
    }
    (new_data, count)
}

fn sort_in_tree(node: &mut Value, folder_id: &str) -> Option<usize> {
    if node["id"].as_str().filter(|s| !s.is_empty()) == Some(folder_id) {
        if let Some(children) = node["children"].as_array_mut() {
            let count = children.len();
            children.sort_by(|a, b| {
                let a_is_folder = a["children"].is_array();
                let b_is_folder = b["children"].is_array();
                b_is_folder.cmp(&a_is_folder).then_with(|| {
                    let an = a["name"].as_str().unwrap_or("");
                    let bn = b["name"].as_str().unwrap_or("");
                    an.cmp(bn)
                })
            });
            return Some(count);
        }
    }
    if let Some(children) = node["children"].as_array_mut() {
        for child in children.iter_mut() {
            if let Some(c) = sort_in_tree(child, folder_id) {
                return Some(c);
            }
        }
    }
    None
}

fn join_path(parent: &str, name: &str) -> String {
    if name.is_empty() { parent.to_string() }
    else if parent.is_empty() { name.to_string() }
    else { format!("{parent}/{name}") }
}

pub fn trunc(s: &str, max: usize) -> String {
    let mut chars = s.chars();
    let out: String = chars.by_ref().take(max).collect();
    if chars.next().is_some() { format!("{out}…") } else { out }
}

fn find_max_id_in(node: &Value) -> u64 {
    let mut max = node["id"].as_str()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(0);
    if let Some(children) = node["children"].as_array() {
        for child in children {
            max = max.max(find_max_id_in(child));
        }
    }
    max
}

fn find_max_id(data: &Value) -> u64 {
    data["roots"].as_object()
        .map(|roots| roots.values().map(find_max_id_in).max().unwrap_or(0))
        .unwrap_or(0)
}

fn count_bookmarks_in(node: &Value) -> usize {
    if node["type"].as_str() == Some("url") { return 1; }
    node["children"].as_array()
        .map(|c| c.iter().map(count_bookmarks_in).sum())
        .unwrap_or(0)
}

/// 统计指定路径文件夹下的书签总数
pub fn count_folder_bookmarks(data: &Value, folder_path: &str) -> usize {
    let parts: Vec<&str> = folder_path.split('/').collect();
    let roots = match data["roots"].as_object() { Some(r) => r, None => return 0 };
    let root_node = match parts[0] {
        "书签栏"    => match roots.get("bookmark_bar") { Some(n) => n, None => return 0 },
        "其他书签"   => match roots.get("other")        { Some(n) => n, None => return 0 },
        "移动设备书签" => match roots.get("synced")      { Some(n) => n, None => return 0 },
        _ => return 0,
    };
    let mut current = root_node;
    for part in parts.iter().skip(1) {
        let children = match current["children"].as_array() { Some(c) => c, None => return 0 };
        current = match children.iter().find(|c| {
            c["name"].as_str() == Some(part) && c["type"].as_str() != Some("url")
        }) { Some(n) => n, None => return 0 };
    }
    count_bookmarks_in(current)
}

/// 在指定路径下新建空文件夹
pub fn create_folder_in_roots(data: &Value, folder_path: &str) -> Result<Value> {
    let idx = folder_path.rfind('/')
        .ok_or_else(|| anyhow::anyhow!("路径须含父文件夹，如 '书签栏/云服务器/新分类'"))?;
    let parent_path = &folder_path[..idx];
    let folder_name = &folder_path[idx + 1..];
    if folder_name.is_empty() { bail!("文件夹名不能为空"); }

    let parent_id = find_folder_id(data, parent_path)
        .ok_or_else(|| anyhow::anyhow!("父文件夹不存在：「{parent_path}」"))?;

    let new_id = (find_max_id(data) + 1).to_string();
    let new_folder = serde_json::json!({
        "children": [],
        "date_added": "0",
        "date_modified": "0",
        "id": new_id,
        "name": folder_name,
        "type": "folder"
    });
    insert_into_roots(data, &parent_id, &[new_folder])
}
