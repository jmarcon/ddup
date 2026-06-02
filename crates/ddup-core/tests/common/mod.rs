#![allow(dead_code)]

use std::{fs, path::Path};

use ddup_core::DirNode;
use tempfile::TempDir;

pub fn build_tree(spec: &[(&str, Option<&[u8]>)]) -> TempDir {
    let dir = TempDir::new().unwrap();
    for (path, content) in spec {
        let full_path = dir.path().join(path);
        if let Some(bytes) = content {
            if let Some(parent) = full_path.parent() {
                fs::create_dir_all(parent).unwrap();
            }
            fs::write(full_path, bytes).unwrap();
        } else {
            fs::create_dir_all(full_path).unwrap();
        }
    }
    dir
}

pub fn assert_dir_count(nodes: &[DirNode], expected: usize) {
    assert_eq!(nodes.len(), expected);
}

pub fn has_path(nodes: &[DirNode], root: &Path, path: &str) -> bool {
    let full_path = root.join(path);
    nodes.iter().any(|node| node.path == full_path)
}
