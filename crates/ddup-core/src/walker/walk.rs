//! Recursive post-order filesystem walk.

use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use walkdir::{DirEntry as WalkDirEntry, WalkDir};

use super::{DirNode, FileEntry, WalkConfig};
use crate::{CoreError, Result};

/// Walks `root` recursively and returns directory nodes in post-order.
pub fn walk(root: &Path, cfg: &WalkConfig) -> Result<Vec<DirNode>> {
    walk_with_errors(root, cfg).map(|(nodes, _errors)| nodes)
}

/// Walks `root` recursively and returns directory nodes plus non-fatal errors.
pub fn walk_with_errors(root: &Path, cfg: &WalkConfig) -> Result<(Vec<DirNode>, Vec<String>)> {
    if !root.exists() {
        return Err(CoreError::PathNotFound(root.to_path_buf()));
    }

    let mut nodes = BTreeMap::<PathBuf, DirNode>::new();
    let mut errors = Vec::new();
    let walker = WalkDir::new(root)
        .follow_links(cfg.follow_symlinks)
        .contents_first(true)
        .into_iter();

    for entry in walker {
        let entry = match entry {
            Ok(entry) => entry,
            Err(error) => {
                errors.push(format!("walk error: {error}"));
                continue;
            }
        };
        if !should_keep(&entry, root, cfg) || path_is_ignored(entry.path(), root, cfg) {
            continue;
        }
        let path = entry.path().to_path_buf();
        if entry.file_type().is_dir() {
            nodes
                .entry(path.clone())
                .or_insert_with(|| DirNode::new(path));
        } else if entry.file_type().is_file() {
            let metadata = match entry.metadata() {
                Ok(metadata) => metadata,
                Err(error) => {
                    errors.push(format!("metadata error at {}: {error}", path.display()));
                    continue;
                }
            };
            let size = metadata.len();
            if size < cfg.min_file_size {
                continue;
            }
            if let Some(parent) = path.parent() {
                let parent_path = parent.to_path_buf();
                let node = nodes
                    .entry(parent_path.clone())
                    .or_insert_with(|| DirNode::new(parent_path));
                node.files.push(FileEntry { path, size });
            }
        }
    }

    let keys = nodes.keys().cloned().collect::<Vec<_>>();
    for path in keys {
        if path == root {
            continue;
        }
        if let Some(parent) = path.parent() {
            if let Some(parent_node) = nodes.get_mut(parent) {
                parent_node.child_dirs.push(path.clone());
            }
        }
    }

    let mut out = nodes.into_values().collect::<Vec<_>>();
    for node in &mut out {
        node.files.sort_by(|a, b| a.path.cmp(&b.path));
        node.child_dirs.sort();
    }
    out.sort_by(|a, b| {
        b.path
            .components()
            .count()
            .cmp(&a.path.components().count())
            .then(a.path.cmp(&b.path))
    });
    Ok((out, errors))
}

fn should_keep(entry: &WalkDirEntry, root: &Path, cfg: &WalkConfig) -> bool {
    if entry.path() == root {
        return true;
    }
    let Some(name) = entry.file_name().to_str() else {
        return true;
    };
    if cfg.ignore_hidden && name.starts_with('.') {
        return false;
    }
    !cfg.ignore_patterns.iter().any(|pattern| pattern == name)
}

fn path_is_ignored(path: &Path, root: &Path, cfg: &WalkConfig) -> bool {
    let Ok(relative) = path.strip_prefix(root) else {
        return false;
    };
    relative.components().any(|component| {
        component
            .as_os_str()
            .to_str()
            .is_some_and(|name| cfg.ignore_patterns.iter().any(|pattern| pattern == name))
    })
}
