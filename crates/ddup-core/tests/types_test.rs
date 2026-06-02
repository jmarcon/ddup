#![allow(missing_docs, clippy::expect_used, clippy::panic, clippy::unwrap_used)]

use std::collections::HashMap;

use std::str::FromStr;

use ddup_core::{DirHash, DupStatus, EntryStatus, FileHash, ScanMode};

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
