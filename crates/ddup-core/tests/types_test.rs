#![allow(missing_docs, clippy::expect_used, clippy::panic, clippy::unwrap_used)]

use std::collections::HashMap;

use std::str::FromStr;

use std::path::PathBuf;

use ddup_core::{
    DirHash, DupEntry, DupFileEntry, DupFileGroup, DupGroup, DupStatus, EntryStatus, FileHash,
    NodeKind, ScanMode, SortBy, SortConfig, SortOrder, TreeStats,
};

#[test]
fn hash_json_round_trip() {
    let dir = DirHash::new("abc");
    let file = FileHash::new("def");

    let dir_json = serde_json::to_string(&dir).unwrap();
    let file_json = serde_json::to_string(&file).unwrap();

    assert_eq!(serde_json::from_str::<DirHash>(&dir_json).unwrap(), dir);
    assert_eq!(serde_json::from_str::<FileHash>(&file_json).unwrap(), file);
}

#[test]
fn hashes_work_as_hash_map_keys() {
    let mut dirs = HashMap::new();
    let mut files = HashMap::new();

    dirs.insert(DirHash::new("same"), 1_u8);
    files.insert(FileHash::new("same"), 2_u8);

    assert_eq!(dirs.get(&DirHash::new("same")), Some(&1));
    assert_eq!(files.get(&FileHash::new("same")), Some(&2));
}

#[test]
fn dir_hash_and_file_hash_are_distinct_types() {
    fn takes_dir_hash(hash: DirHash) -> String {
        hash.into_inner()
    }

    assert_eq!(takes_dir_hash(DirHash::new("abc")), "abc");
}

#[test]
fn entry_status_round_trip() {
    let status = EntryStatus::from_str("pending").unwrap();

    assert_eq!(status, EntryStatus::Pending);
    assert_eq!(status.to_string(), "pending");
    assert_eq!(serde_json::to_string(&status).unwrap(), "\"pending\"");
}

#[test]
fn dup_status_round_trip() {
    let status = DupStatus::from_str("dup_dir").unwrap();

    assert_eq!(status, DupStatus::DupDir);
    assert_eq!(status.to_string(), "dup_dir");
    assert_eq!(serde_json::to_string(&status).unwrap(), "\"dup_dir\"");
}

#[test]
fn scan_mode_round_trip() {
    let mode = ScanMode::from_str("smart").unwrap();

    assert_eq!(mode, ScanMode::Smart);
    assert_eq!(mode.to_string(), "smart");
}

#[test]
fn scan_mode_defaults_to_smart() {
    assert_eq!(ScanMode::default(), ScanMode::Smart);
}

#[test]
fn dup_group_round_trip() {
    let group = DupGroup {
        id: Some(1),
        dir_hash: DirHash::new("dir"),
        file_count: 2,
        size_bytes: 10,
        entries: vec![DupEntry {
            id: Some(2),
            path: PathBuf::from("a"),
            status: EntryStatus::Pending,
            dir_hash: DirHash::new("dir"),
        }],
    };

    let json = serde_json::to_string(&group).unwrap();

    assert_eq!(serde_json::from_str::<DupGroup>(&json).unwrap(), group);
}

#[test]
fn dup_file_group_round_trip() {
    let group = DupFileGroup {
        id: Some(1),
        file_hash: FileHash::new("file"),
        size_bytes: 10,
        entries: vec![DupFileEntry {
            id: Some(2),
            path: PathBuf::from("a.txt"),
            status: EntryStatus::Pending,
            suppressed_by_dir_group: Some(9),
        }],
    };

    let json = serde_json::to_string(&group).unwrap();

    assert_eq!(serde_json::from_str::<DupFileGroup>(&json).unwrap(), group);
}

#[test]
fn dup_models_do_not_emit_unknown_fields() {
    let group = DupFileGroup {
        id: None,
        file_hash: FileHash::new("file"),
        size_bytes: 10,
        entries: Vec::new(),
    };
    let value = serde_json::to_value(group).unwrap();
    let object = value.as_object().unwrap();

    assert!(object.contains_key("id"));
    assert!(object.contains_key("file_hash"));
    assert!(object.contains_key("size_bytes"));
    assert!(object.contains_key("entries"));
    assert_eq!(object.len(), 4);
}

#[test]
fn tree_stats_round_trip() {
    let stats = TreeStats {
        path: PathBuf::from("root/a.txt"),
        kind: NodeKind::File,
        parent_path: Some(PathBuf::from("root")),
        depth: 1,
        size_bytes: 5,
        size_recursive: 5,
        file_count_recursive: 1,
        extension: Some("txt".to_owned()),
        dup_status: DupStatus::DupFile,
        dir_group_id: None,
        file_group_id: Some(3),
        wasted_bytes: 5,
        content_hash: Some("file-hash".to_owned()),
    };

    let json = serde_json::to_string(&stats).unwrap();

    assert_eq!(serde_json::from_str::<TreeStats>(&json).unwrap(), stats);
}

#[test]
fn tree_stats_optional_fields_serialize_as_null() {
    let stats = TreeStats {
        path: PathBuf::from("root"),
        kind: NodeKind::Dir,
        parent_path: None,
        depth: 0,
        size_bytes: 0,
        size_recursive: 0,
        file_count_recursive: 0,
        extension: None,
        dup_status: DupStatus::Unique,
        dir_group_id: None,
        file_group_id: None,
        wasted_bytes: 0,
        content_hash: None,
    };

    let value = serde_json::to_value(stats).unwrap();

    assert!(value.get("parent_path").unwrap().is_null());
    assert!(value.get("extension").unwrap().is_null());
    assert!(value.get("dir_group_id").unwrap().is_null());
    assert!(value.get("file_group_id").unwrap().is_null());
    assert!(value.get("content_hash").unwrap().is_null());
}

#[test]
fn sort_config_default() {
    let sort = SortConfig::default();

    assert_eq!(sort.by, SortBy::TotalSize);
    assert_eq!(sort.order, SortOrder::Desc);
}

#[test]
fn sort_config_next_by_cycles() {
    let mut sort = SortConfig::default();

    sort.next_by();
    assert_eq!(sort.by, SortBy::FileCount);
    sort.next_by();
    assert_eq!(sort.by, SortBy::Name);
    sort.next_by();
    assert_eq!(sort.by, SortBy::TotalSize);

    sort.toggle_order();
    assert_eq!(sort.order, SortOrder::Asc);
}
