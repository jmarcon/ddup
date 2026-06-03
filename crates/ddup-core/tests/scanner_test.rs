#![allow(missing_docs, clippy::expect_used, clippy::panic, clippy::unwrap_used)]

mod common;

use std::sync::mpsc;

use ddup_core::{
    DirHash, DupStatus, FileHash, NodeKind, ScanEvent, ScanMode, WalkConfig, scan, scan_roots,
};

use common::build_tree;

#[test]
fn s1_two_identical_dirs_create_one_group() {
    let dir = build_tree(&[("a/x.txt", Some(b"1")), ("b/x.txt", Some(b"1"))]);
    let result = scan(dir.path(), &WalkConfig::default(), ScanMode::Smart, None).unwrap();

    assert_eq!(result.dir_groups.len(), 1);
    assert_eq!(result.dir_groups[0].entries.len(), 2);
}

#[test]
fn s2_three_dirs_two_matching_create_one_group() {
    let dir = build_tree(&[
        ("a/x.txt", Some(b"1")),
        ("b/x.txt", Some(b"1")),
        ("c/x.txt", Some(b"2")),
    ]);
    let result = scan(dir.path(), &WalkConfig::default(), ScanMode::Smart, None).unwrap();

    assert_eq!(result.dir_groups.len(), 1);
}

#[test]
fn s3_two_distinct_groups() {
    let dir = build_tree(&[
        ("a/x.txt", Some(b"1")),
        ("b/x.txt", Some(b"1")),
        ("c/y.txt", Some(b"2")),
        ("d/y.txt", Some(b"2")),
    ]);
    let result = scan(dir.path(), &WalkConfig::default(), ScanMode::Smart, None).unwrap();

    assert_eq!(result.dir_groups.len(), 2);
}

#[test]
fn s4_no_duplicates_is_empty() {
    let dir = build_tree(&[("a/x.txt", Some(b"1")), ("b/x.txt", Some(b"2"))]);
    let result = scan(dir.path(), &WalkConfig::default(), ScanMode::Smart, None).unwrap();

    assert!(result.dir_groups.is_empty());
}

#[test]
fn s5_wasted_bytes_for_directory_group() {
    let dir = build_tree(&[("a/x.txt", Some(b"1234")), ("b/x.txt", Some(b"1234"))]);
    let result = scan(dir.path(), &WalkConfig::default(), ScanMode::Smart, None).unwrap();

    assert_eq!(result.dir_groups[0].size_bytes, 4);
    assert!(result.summary.wasted_bytes >= 4);
}

#[test]
fn s6_scan_is_deterministic_for_groups() {
    let dir = build_tree(&[("a/x.txt", Some(b"1")), ("b/x.txt", Some(b"1"))]);
    let mut paths = Vec::new();
    for _ in 0..5 {
        let result = scan(dir.path(), &WalkConfig::default(), ScanMode::Smart, None).unwrap();
        paths.push(result.dir_groups[0].entries[0].path.clone());
    }

    assert!(paths.windows(2).all(|pair| pair[0] == pair[1]));
}

#[test]
fn s7_progress_reports_total_dirs() {
    let dir = build_tree(&[("a/x.txt", Some(b"1")), ("b/x.txt", Some(b"1"))]);
    let (tx, rx) = mpsc::channel();
    let result = scan(
        dir.path(),
        &WalkConfig::default(),
        ScanMode::Smart,
        Some(tx),
    )
    .unwrap();
    let events = rx.try_iter().collect::<Vec<_>>();

    assert!(events.iter().any(|event| matches!(
        event,
        ScanEvent::Started {
            total_dirs_estimate
        } if u64::try_from(*total_dirs_estimate).is_ok_and(|value| value == result.summary.total_dirs)
    )));
}

#[test]
fn fd1_identical_files_create_group() {
    let dir = build_tree(&[("a.txt", Some(b"1")), ("b.txt", Some(b"1"))]);
    let result = scan(dir.path(), &WalkConfig::default(), ScanMode::Flat, None).unwrap();

    assert_eq!(result.file_groups.len(), 1);
}

#[test]
fn fd2_unique_files_no_group() {
    let dir = build_tree(&[("a.txt", Some(b"1")), ("b.txt", Some(b"2"))]);
    let result = scan(dir.path(), &WalkConfig::default(), ScanMode::Flat, None).unwrap();

    assert!(result.file_groups.is_empty());
}

#[test]
fn fd3_three_identical_files_single_group() {
    let dir = build_tree(&[
        ("a.txt", Some(b"1")),
        ("b.txt", Some(b"1")),
        ("c.txt", Some(b"1")),
    ]);
    let result = scan(dir.path(), &WalkConfig::default(), ScanMode::Flat, None).unwrap();

    assert_eq!(result.file_groups[0].entries.len(), 3);
}

#[test]
fn fd4_smart_suppresses_files_inside_duplicate_dirs() {
    let dir = build_tree(&[("a/x.txt", Some(b"1")), ("b/x.txt", Some(b"1"))]);
    let result = scan(dir.path(), &WalkConfig::default(), ScanMode::Smart, None).unwrap();

    assert!(
        result.file_groups[0]
            .entries
            .iter()
            .all(|entry| entry.suppressed_by_dir_group.is_some())
    );
}

#[test]
fn fd5_flat_does_not_suppress() {
    let dir = build_tree(&[("a/x.txt", Some(b"1")), ("b/x.txt", Some(b"1"))]);
    let result = scan(dir.path(), &WalkConfig::default(), ScanMode::Flat, None).unwrap();

    assert!(
        result.file_groups[0]
            .entries
            .iter()
            .all(|entry| entry.suppressed_by_dir_group.is_none())
    );
}

#[test]
fn fd6_mixed_dir_and_file_groups() {
    let dir = build_tree(&[
        ("a/x.txt", Some(b"1")),
        ("b/x.txt", Some(b"1")),
        ("c.txt", Some(b"2")),
        ("d.txt", Some(b"2")),
    ]);
    let result = scan(dir.path(), &WalkConfig::default(), ScanMode::Smart, None).unwrap();

    assert_eq!(result.dir_groups.len(), 1);
    assert!(result.file_groups.iter().any(|group| {
        group
            .entries
            .iter()
            .all(|entry| entry.suppressed_by_dir_group.is_none())
    }));
}

#[test]
fn fd7_file_groups_are_deterministic() {
    let dir = build_tree(&[("a.txt", Some(b"1")), ("b.txt", Some(b"1"))]);
    let mut first_paths = Vec::new();
    for _ in 0..5 {
        let result = scan(dir.path(), &WalkConfig::default(), ScanMode::Flat, None).unwrap();
        first_paths.push(result.file_groups[0].entries[0].path.clone());
    }

    assert!(first_paths.windows(2).all(|pair| pair[0] == pair[1]));
}

#[test]
fn fd8_wasted_bytes_differs_by_mode() {
    let dir = build_tree(&[("a/x.txt", Some(b"1")), ("b/x.txt", Some(b"1"))]);
    let smart = scan(dir.path(), &WalkConfig::default(), ScanMode::Smart, None).unwrap();
    let flat = scan(dir.path(), &WalkConfig::default(), ScanMode::Flat, None).unwrap();

    assert!(flat.summary.wasted_bytes >= smart.summary.wasted_bytes);
}

#[test]
fn ts1_tree_nodes_cover_dirs_and_files() {
    let dir = build_tree(&[
        ("a/1.txt", Some(b"1")),
        ("a/2.txt", Some(b"2")),
        ("b/3.txt", Some(b"3")),
        ("c/4.txt", Some(b"4")),
        ("c/5.txt", Some(b"5")),
    ]);
    let result = scan(dir.path(), &WalkConfig::default(), ScanMode::Smart, None).unwrap();

    assert_eq!(result.tree_stats.len(), 9);
}

#[test]
fn ts2_root_recursive_size_is_sum() {
    let dir = build_tree(&[("a.txt", Some(b"12")), ("b.txt", Some(b"123"))]);
    let result = scan(dir.path(), &WalkConfig::default(), ScanMode::Smart, None).unwrap();
    let root = result
        .tree_stats
        .iter()
        .find(|node| node.path == dir.path())
        .unwrap();

    assert_eq!(root.size_recursive, 5);
}

#[test]
fn ts3_file_count_recursive_is_correct() {
    let dir = build_tree(&[("a/1.txt", Some(b"1")), ("a/2.txt", Some(b"2"))]);
    let result = scan(dir.path(), &WalkConfig::default(), ScanMode::Smart, None).unwrap();
    let a = result
        .tree_stats
        .iter()
        .find(|node| node.path == dir.path().join("a"))
        .unwrap();

    assert_eq!(a.file_count_recursive, 2);
}

#[test]
fn ts4_duplicate_dir_status_and_waste() {
    let dir = build_tree(&[("a/x.txt", Some(b"1")), ("b/x.txt", Some(b"1"))]);
    let result = scan(dir.path(), &WalkConfig::default(), ScanMode::Smart, None).unwrap();

    assert!(result.tree_stats.iter().any(|node| {
        node.kind == NodeKind::Dir && node.dup_status == DupStatus::DupDir && node.wasted_bytes > 0
    }));
}

#[test]
fn ts5_duplicate_file_status() {
    let dir = build_tree(&[("a.txt", Some(b"1")), ("b.txt", Some(b"1"))]);
    let result = scan(dir.path(), &WalkConfig::default(), ScanMode::Flat, None).unwrap();

    assert!(
        result
            .tree_stats
            .iter()
            .any(|node| node.dup_status == DupStatus::DupFile)
    );
}

#[test]
fn ts6_partial_dir_status() {
    let dir = build_tree(&[("a/1.txt", Some(b"1")), ("a/2.txt", Some(b"1"))]);
    let result = scan(dir.path(), &WalkConfig::default(), ScanMode::Flat, None).unwrap();
    let a = result
        .tree_stats
        .iter()
        .find(|node| node.path == dir.path().join("a"))
        .unwrap();

    assert_eq!(a.dup_status, DupStatus::Partial);
}

#[test]
fn ts7_extension_lowercase_and_none() {
    let dir = build_tree(&[("a.JPG", Some(b"1")), ("noext", Some(b"2"))]);
    let result = scan(dir.path(), &WalkConfig::default(), ScanMode::Smart, None).unwrap();
    let jpg = result
        .tree_stats
        .iter()
        .find(|node| node.path == dir.path().join("a.JPG"))
        .unwrap();
    let noext = result
        .tree_stats
        .iter()
        .find(|node| node.path == dir.path().join("noext"))
        .unwrap();

    assert_eq!(jpg.extension.as_deref(), Some("jpg"));
    assert_eq!(noext.extension, None);
}

#[test]
fn ts8_parent_path_root_is_none() {
    let dir = build_tree(&[("a/x.txt", Some(b"1"))]);
    let result = scan(dir.path(), &WalkConfig::default(), ScanMode::Smart, None).unwrap();
    let root = result
        .tree_stats
        .iter()
        .find(|node| node.path == dir.path())
        .unwrap();
    let child = result
        .tree_stats
        .iter()
        .find(|node| node.path == dir.path().join("a"))
        .unwrap();

    assert_eq!(root.parent_path, None);
    assert_eq!(child.parent_path.as_deref(), Some(dir.path()));
}

#[test]
fn ts9_every_tree_node_stores_content_hash() {
    let dir = build_tree(&[("a/x.txt", Some(b"1")), ("a/y.txt", Some(b"2"))]);
    let result = scan(dir.path(), &WalkConfig::default(), ScanMode::Smart, None).unwrap();

    assert!(result.tree_stats.iter().all(|node| {
        node.content_hash
            .as_deref()
            .is_some_and(|hash| hash.len() == 64)
    }));
}

#[test]
fn ts10_parent_hash_uses_direct_file_and_direct_child_dir_hashes() {
    let dir = build_tree(&[("root.txt", Some(b"r")), ("a/x.txt", Some(b"x"))]);
    let result = scan(dir.path(), &WalkConfig::default(), ScanMode::Smart, None).unwrap();
    let root = result
        .tree_stats
        .iter()
        .find(|node| node.path == dir.path())
        .unwrap();
    let file = result
        .tree_stats
        .iter()
        .find(|node| node.path == dir.path().join("root.txt"))
        .unwrap();
    let child_dir = result
        .tree_stats
        .iter()
        .find(|node| node.path == dir.path().join("a"))
        .unwrap();
    let expected = ddup_core::hash_dir(
        &[FileHash::new(file.content_hash.clone().unwrap())],
        &[DirHash::new(child_dir.content_hash.clone().unwrap())],
    );

    assert_eq!(root.content_hash.as_deref(), Some(expected.as_str()));
}

#[test]
fn mr1_multi_root_reports_duplicate_dirs_only_between_roots() {
    let left = build_tree(&[
        ("cross/x.txt", Some(b"same")),
        ("local_a/y.txt", Some(b"local")),
        ("local_b/y.txt", Some(b"local")),
    ]);
    let right = build_tree(&[("cross/x.txt", Some(b"same"))]);

    let result = scan_roots(
        &[left.path().to_path_buf(), right.path().to_path_buf()],
        &WalkConfig::default(),
        ScanMode::Smart,
        None,
    )
    .unwrap();

    assert!(result.dir_groups.iter().any(|group| {
        group
            .entries
            .iter()
            .any(|entry| entry.path.starts_with(left.path()))
            && group
                .entries
                .iter()
                .any(|entry| entry.path.starts_with(right.path()))
    }));
    assert!(!result.dir_groups.iter().any(|group| {
        group
            .entries
            .iter()
            .all(|entry| entry.path.starts_with(left.path()))
    }));
}

#[test]
fn mr2_multi_root_reports_duplicate_files_only_between_roots() {
    let left = build_tree(&[
        ("cross.txt", Some(b"same")),
        ("local_a.txt", Some(b"local")),
        ("local_b.txt", Some(b"local")),
    ]);
    let right = build_tree(&[("cross.txt", Some(b"same"))]);

    let result = scan_roots(
        &[left.path().to_path_buf(), right.path().to_path_buf()],
        &WalkConfig::default(),
        ScanMode::Flat,
        None,
    )
    .unwrap();

    assert!(result.file_groups.iter().any(|group| {
        group
            .entries
            .iter()
            .any(|entry| entry.path.starts_with(left.path()))
            && group
                .entries
                .iter()
                .any(|entry| entry.path.starts_with(right.path()))
    }));
    assert!(!result.file_groups.iter().any(|group| {
        group
            .entries
            .iter()
            .all(|entry| entry.path.starts_with(left.path()))
    }));
    assert!(
        result
            .summary
            .root_path
            .display()
            .to_string()
            .contains(" | ")
    );
}
