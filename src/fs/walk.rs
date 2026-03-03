use std::collections::HashMap;
use std::path::{Path, PathBuf};

use ignore::WalkBuilder;
use serde::{Deserialize, Serialize};

use crate::config::FilesystemConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileNode {
    pub name: String,
    pub path: String,
    #[serde(rename = "type")]
    pub node_type: NodeType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<FileNode>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum NodeType {
    File,
    Dir,
}

/// Walk the filesystem from `root`, respecting .gitignore and config excludes.
/// Returns a sorted tree structure.
pub fn walk_tree(root: &Path, config: &FilesystemConfig) -> Vec<FileNode> {
    let mut builder = WalkBuilder::new(root);
    builder
        .hidden(!config.show_hidden)
        .git_ignore(true)
        .git_global(true)
        .git_exclude(true)
        .max_depth(None);

    // Note: builder.add_ignore() expects a gitignore-format FILE path, not a
    // pattern string.  Exclude patterns are applied manually below via the
    // per-entry name check against config.exclude_patterns.

    // Collect all entries
    let mut dirs: HashMap<PathBuf, Vec<FileNode>> = HashMap::new();
    let mut all_dirs: Vec<PathBuf> = Vec::new();

    for entry in builder.build().flatten() {
        let entry_path = entry.path();

        // Skip the root directory itself
        if entry_path == root {
            all_dirs.push(entry_path.to_path_buf());
            continue;
        }

        let relative = match entry_path.strip_prefix(root) {
            Ok(r) => r,
            Err(_) => continue,
        };

        // Skip if any path component matches an exclude pattern
        let dominated = relative.components().any(|c| {
            let name = c.as_os_str().to_str().unwrap_or("");
            config.exclude_patterns.iter().any(|p| p == name)
        });
        if dominated {
            continue;
        }

        let name = relative
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");

        let parent = entry_path.parent().unwrap_or(root).to_path_buf();
        let rel_str = relative.to_string_lossy().replace('\\', "/");

        let node = if entry_path.is_dir() {
            all_dirs.push(entry_path.to_path_buf());
            FileNode {
                name: name.to_string(),
                path: rel_str,
                node_type: NodeType::Dir,
                children: Some(Vec::new()),
            }
        } else {
            FileNode {
                name: name.to_string(),
                path: rel_str,
                node_type: NodeType::File,
                children: None,
            }
        };

        dirs.entry(parent).or_default().push(node);
    }

    // Sort each directory's children: folders first, then alphabetical
    for children in dirs.values_mut() {
        children.sort_by(|a, b| {
            match (&a.node_type, &b.node_type) {
                (NodeType::Dir, NodeType::File) => std::cmp::Ordering::Less,
                (NodeType::File, NodeType::Dir) => std::cmp::Ordering::Greater,
                _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            }
        });
    }

    // Build tree bottom-up: assign children to their parent dirs
    // Process deepest paths first
    all_dirs.sort_by(|a, b| b.components().count().cmp(&a.components().count()));

    for dir_path in &all_dirs {
        if dir_path == root {
            continue;
        }
        let collected = dirs.remove(dir_path).unwrap_or_default();
        let parent = dir_path.parent().unwrap_or(root).to_path_buf();
        if let Some(siblings) = dirs.get_mut(&parent) {
            if let Some(node) = siblings.iter_mut().find(|n| {
                n.node_type == NodeType::Dir && dir_path.ends_with(&n.name)
            }) {
                node.children = Some(collected);
            }
        }
    }

    dirs.remove(&root.to_path_buf()).unwrap_or_default()
}

/// Check if a node exists in the tree at the given relative path.
pub fn node_exists(tree: &[FileNode], relative_path: &str) -> bool {
    let parts: Vec<&str> = relative_path.split('/').collect();
    exists_recursive(tree, &parts)
}

fn exists_recursive(nodes: &[FileNode], parts: &[&str]) -> bool {
    if parts.is_empty() {
        return false;
    }
    for node in nodes {
        if node.name == parts[0] {
            if parts.len() == 1 {
                return true;
            }
            if let Some(ref children) = node.children {
                return exists_recursive(children, &parts[1..]);
            }
        }
    }
    false
}

/// Insert a node into the tree at the given relative path.
pub fn insert_node(tree: &mut Vec<FileNode>, relative_path: &str, is_dir: bool) {
    let parts: Vec<&str> = relative_path.split('/').collect();
    insert_recursive(tree, &parts, relative_path, is_dir);
    sort_children(tree);
}

fn insert_recursive(nodes: &mut Vec<FileNode>, parts: &[&str], full_path: &str, is_dir: bool) {
    if parts.is_empty() {
        return;
    }

    if parts.len() == 1 {
        // Leaf — add if not present
        if !nodes.iter().any(|n| n.name == parts[0]) {
            nodes.push(FileNode {
                name: parts[0].to_string(),
                path: full_path.to_string(),
                node_type: if is_dir { NodeType::Dir } else { NodeType::File },
                children: if is_dir { Some(Vec::new()) } else { None },
            });
        }
        return;
    }

    // Find or create the intermediate directory
    let dir_name = parts[0];
    let dir_node = if let Some(pos) = nodes.iter().position(|n| n.name == dir_name && n.node_type == NodeType::Dir) {
        &mut nodes[pos]
    } else {
        // Compute correct path based on depth.
        // full_path has N segments; parts has M remaining.  The directory we're
        // creating is at depth (N - M + 1) from the root.
        let all_segments: Vec<&str> = full_path.split('/').collect();
        let depth = all_segments.len() - parts.len() + 1;
        let dir_path = all_segments[..depth].join("/");
        nodes.push(FileNode {
            name: dir_name.to_string(),
            path: dir_path,
            node_type: NodeType::Dir,
            children: Some(Vec::new()),
        });
        nodes.last_mut().unwrap()
    };

    if let Some(ref mut children) = dir_node.children {
        insert_recursive(children, &parts[1..], full_path, is_dir);
        sort_children(children);
    }
}

/// Remove a node from the tree at the given relative path.
pub fn remove_node(tree: &mut Vec<FileNode>, relative_path: &str) -> bool {
    let parts: Vec<&str> = relative_path.split('/').collect();
    remove_recursive(tree, &parts)
}

fn remove_recursive(nodes: &mut Vec<FileNode>, parts: &[&str]) -> bool {
    if parts.is_empty() {
        return false;
    }

    if parts.len() == 1 {
        let len_before = nodes.len();
        nodes.retain(|n| n.name != parts[0]);
        return nodes.len() != len_before;
    }

    for node in nodes.iter_mut() {
        if node.name == parts[0] && node.node_type == NodeType::Dir {
            if let Some(ref mut children) = node.children {
                return remove_recursive(children, &parts[1..]);
            }
        }
    }

    false
}

fn sort_children(nodes: &mut Vec<FileNode>) {
    nodes.sort_by(|a, b| {
        match (&a.node_type, &b.node_type) {
            (NodeType::Dir, NodeType::File) => std::cmp::Ordering::Less,
            (NodeType::File, NodeType::Dir) => std::cmp::Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        }
    });
}
