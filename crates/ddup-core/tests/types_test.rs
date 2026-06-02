#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

use std::collections::HashMap;

use ddup_core::{DirHash, FileHash};

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

