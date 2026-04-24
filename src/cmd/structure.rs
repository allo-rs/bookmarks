use std::path::Path;
use anyhow::Result;
use crate::bookmark::{self, Node};

pub fn run(path: &Path) -> Result<()> {
    let file = bookmark::load(path)?;
    let all = bookmark::get_all_bookmarks(&file);
    println!("总书签数: {}\n", all.len());
    print_tree(&file.roots.bookmark_bar, 0);
    Ok(())
}

fn print_tree(node: &Node, depth: usize) {
    let name = node.name.as_deref().unwrap_or("");
    let Some(children) = &node.children else { return };
    let urls = children.iter().filter(|c| c.kind.as_deref() == Some("url")).count();
    let folders: Vec<_> = children.iter().filter(|c| c.children.is_some()).collect();

    if depth > 0 && !name.is_empty() {
        let indent = "  ".repeat(depth);
        println!("{indent}📁 {name}  ({urls} 书签, {} 子文件夹)", folders.len());
    }
    for f in folders {
        print_tree(f, depth + 1);
    }
}
