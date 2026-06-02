#![allow(missing_docs, clippy::expect_used, clippy::panic, clippy::unwrap_used)]

mod common;

use ddup_core::{CoreError, WalkConfig, walk};

use common::{assert_dir_count, build_tree, has_path};

#[test]
fn w1_flat_one_level_counts_root_and_dirs() {
    let dir = build_tree(&[("a", None), ("b", None), ("c", None)]);
    let nodes = walk(dir.path(), &WalkConfig::default()).unwrap();

    assert_dir_count(&nodes, 4);
}

#[test]
fn w2_four_levels_are_visited() {
    let dir = build_tree(&[("a/b/c/d", None)]);
    let nodes = walk(dir.path(), &WalkConfig::default()).unwrap();

    assert!(has_path(&nodes, dir.path(), "a/b/c/d"));
}

#[test]
fn w3_empty_dir_is_included() {
    let dir = build_tree(&[("empty", None)]);
    let nodes = walk(dir.path(), &WalkConfig::default()).unwrap();

    assert!(has_path(&nodes, dir.path(), "empty"));
}

#[test]
fn w4_symlink_dir_is_not_followed_by_default() {
    let dir = build_tree(&[("real/child", None)]);
    #[cfg(windows)]
    std::os::windows::fs::symlink_dir(dir.path().join("real"), dir.path().join("link")).unwrap();
    #[cfg(unix)]
    std::os::unix::fs::symlink(dir.path().join("real"), dir.path().join("link")).unwrap();

    let nodes = walk(dir.path(), &WalkConfig::default()).unwrap();

    assert!(!has_path(&nodes, dir.path(), "link/child"));
}

#[test]
fn w5_node_modules_is_ignored() {
    let dir = build_tree(&[("node_modules/pkg", None), ("src", None)]);
    let nodes = walk(dir.path(), &WalkConfig::default()).unwrap();

    assert!(!has_path(&nodes, dir.path(), "node_modules"));
    assert!(has_path(&nodes, dir.path(), "src"));
}

#[test]
fn w6_post_order_places_child_before_parent() {
    let dir = build_tree(&[("a/b", None)]);
    let nodes = walk(dir.path(), &WalkConfig::default()).unwrap();
    let child = nodes
        .iter()
        .position(|node| node.path == dir.path().join("a/b"))
        .unwrap();
    let parent = nodes
        .iter()
        .position(|node| node.path == dir.path().join("a"))
        .unwrap();

    assert!(child < parent);
}

#[test]
fn w7_missing_root_returns_path_not_found() {
    let result = walk(std::path::Path::new("missing-root"), &WalkConfig::default());

    assert!(matches!(result, Err(CoreError::PathNotFound(_))));
}

#[test]
fn w8_walk_continues_when_subdir_cannot_be_read() {
    let dir = build_tree(&[("ok", None), ("blocked", None)]);
    let nodes = walk(dir.path(), &WalkConfig::default()).unwrap();

    assert!(has_path(&nodes, dir.path(), "ok"));
}
