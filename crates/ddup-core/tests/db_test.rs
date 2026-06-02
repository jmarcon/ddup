#![allow(missing_docs, clippy::expect_used, clippy::panic, clippy::unwrap_used)]

use std::{fs, time::Instant};

use chrono::Utc;
use ddup_core::{
    Db, DirHash, DupEntry, DupFileEntry, DupFileGroup, DupGroup, DupStatus, EntryStatus, FileHash,
    NodeKind, Scan, ScanMode, SortBy, SortConfig, SortOrder, TreeStats,
};
use tempfile::TempDir;

fn scan(root: &str, mode: ScanMode) -> Scan {
    Scan {
        id: None,
        root_path: root.into(),
        scanned_at: Utc::now(),
        scan_mode: mode,
        total_dirs: 2,
        total_files: 3,
        wasted_bytes: 4,
    }
}

fn dir_group(name: &str, size: u64, count: u64) -> DupGroup {
    DupGroup {
        id: None,
        dir_hash: DirHash::new(format!("hash-{name}")),
        file_count: count,
        size_bytes: size,
        entries: vec![
            DupEntry {
                id: None,
                path: format!("{name}/a").into(),
                status: EntryStatus::Pending,
                dir_hash: DirHash::new(format!("hash-{name}")),
            },
            DupEntry {
                id: None,
                path: format!("{name}/b").into(),
                status: EntryStatus::Pending,
                dir_hash: DirHash::new(format!("hash-{name}")),
            },
        ],
    }
}

fn file_group(name: &str, suppressed: bool) -> DupFileGroup {
    DupFileGroup {
        id: None,
        file_hash: FileHash::new(format!("file-{name}")),
        size_bytes: 10,
        entries: vec![
            DupFileEntry {
                id: None,
                path: format!("{name}/a.txt").into(),
                status: EntryStatus::Pending,
                suppressed_by_dir_group: suppressed.then_some(1),
            },
            DupFileEntry {
                id: None,
                path: format!("{name}/b.txt").into(),
                status: EntryStatus::Pending,
                suppressed_by_dir_group: None,
            },
        ],
    }
}

fn tree(path: &str, parent: Option<&str>, kind: NodeKind, depth: u32, waste: u64) -> TreeStats {
    TreeStats {
        path: path.into(),
        parent_path: parent.map(Into::into),
        kind,
        depth,
        size_bytes: if kind == NodeKind::File { 1 } else { 0 },
        size_recursive: 1,
        file_count_recursive: 1,
        extension: (kind == NodeKind::File).then_some("txt".to_owned()),
        dup_status: if waste > 0 {
            DupStatus::DupFile
        } else {
            DupStatus::Unique
        },
        dir_group_id: None,
        file_group_id: None,
        wasted_bytes: waste,
    }
}

#[test]
fn d1_new_database_applies_v2() {
    assert!(Db::memory().is_ok());
}

#[test]
fn d2_reopen_database_keeps_schema() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("x.db");
    Db::open(&path).unwrap();

    assert!(Db::open(&path).unwrap().list_scans().unwrap().is_empty());
}

#[test]
fn d3_user_version_zero_migrates() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("x.db");
    rusqlite::Connection::open(&path).unwrap();

    assert!(Db::open(&path).is_ok());
}

#[test]
fn d4_scan_round_trip() {
    let db = Db::memory().unwrap();
    let id = db.upsert_scan(&scan("root", ScanMode::Smart)).unwrap();
    let fetched = db.fetch_scan("root".as_ref()).unwrap().unwrap();

    assert_eq!(fetched.id, Some(id));
}

#[test]
fn d5_rescan_replaces() {
    let db = Db::memory().unwrap();
    db.upsert_scan(&scan("root", ScanMode::Smart)).unwrap();
    db.upsert_scan(&scan("root", ScanMode::Flat)).unwrap();

    assert_eq!(db.list_scans().unwrap().len(), 1);
    assert_eq!(
        db.fetch_scan("root".as_ref()).unwrap().unwrap().scan_mode,
        ScanMode::Flat
    );
}

#[test]
fn d6_distinct_scan_paths_are_isolated() {
    let db = Db::memory().unwrap();
    db.upsert_scan(&scan("a", ScanMode::Smart)).unwrap();
    db.upsert_scan(&scan("b", ScanMode::Smart)).unwrap();

    assert_eq!(db.list_scans().unwrap().len(), 2);
}

#[test]
fn d7_scan_mode_persisted() {
    let db = Db::memory().unwrap();
    db.upsert_scan(&scan("root", ScanMode::Flat)).unwrap();

    assert_eq!(db.last_scan().unwrap().unwrap().scan_mode, ScanMode::Flat);
}

#[test]
fn d8_d9_dir_group_and_entries_round_trip() {
    let mut db = Db::memory().unwrap();
    let scan_id = db.upsert_scan(&scan("root", ScanMode::Smart)).unwrap();
    db.insert_dir_groups(scan_id, &[dir_group("b", 20, 2)])
        .unwrap();
    let groups = db.fetch_dir_groups(scan_id, SortConfig::default()).unwrap();

    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].entries.len(), 2);
}

#[test]
fn d10_d11_d12_dir_sorting() {
    let mut db = Db::memory().unwrap();
    let scan_id = db.upsert_scan(&scan("root", ScanMode::Smart)).unwrap();
    db.insert_dir_groups(scan_id, &[dir_group("b", 20, 3), dir_group("a", 10, 2)])
        .unwrap();

    assert_eq!(
        db.fetch_dir_groups(scan_id, SortConfig::default()).unwrap()[0].entries[0].path,
        std::path::PathBuf::from("a/a")
    );
    assert_eq!(
        db.fetch_dir_groups(
            scan_id,
            SortConfig {
                by: SortBy::TotalSize,
                order: SortOrder::Desc,
            },
        )
        .unwrap()[0]
            .size_bytes,
        20
    );
    assert_eq!(
        db.fetch_dir_groups(
            scan_id,
            SortConfig {
                by: SortBy::FileCount,
                order: SortOrder::Asc,
            },
        )
        .unwrap()[0]
            .file_count,
        2
    );
}

#[test]
fn d13_d14_d15_file_groups_filtering() {
    let mut db = Db::memory().unwrap();
    let scan_id = db.upsert_scan(&scan("root", ScanMode::Smart)).unwrap();
    db.insert_dir_groups(scan_id, &[dir_group("d", 1, 1)])
        .unwrap();
    db.insert_file_groups(scan_id, &[file_group("x", true)])
        .unwrap();

    assert_eq!(
        db.fetch_file_groups(scan_id, SortConfig::default(), false)
            .unwrap()[0]
            .entries
            .len(),
        2
    );
    assert!(
        db.fetch_file_groups(scan_id, SortConfig::default(), true)
            .unwrap()
            .is_empty()
    );
}

#[test]
fn d16_d17_tree_nodes_round_trip_and_cascade() {
    let mut db = Db::memory().unwrap();
    let scan_id = db.upsert_scan(&scan("root", ScanMode::Smart)).unwrap();
    db.insert_tree_nodes(scan_id, &[tree("root", None, NodeKind::Dir, 0, 0)])
        .unwrap();
    assert!(db.fetch_node(scan_id, "root".as_ref()).unwrap().is_some());
    db.delete_scan(scan_id).unwrap();
    assert!(db.fetch_node(scan_id, "root".as_ref()).unwrap().is_none());
}

#[test]
fn d18_unicode_tree_path() {
    let mut db = Db::memory().unwrap();
    let scan_id = db
        .upsert_scan(&scan("日本語/café", ScanMode::Smart))
        .unwrap();
    db.insert_tree_nodes(scan_id, &[tree("日本語/café", None, NodeKind::Dir, 0, 0)])
        .unwrap();

    assert!(
        db.fetch_node(scan_id, "日本語/café".as_ref())
            .unwrap()
            .is_some()
    );
}

#[test]
fn d19_d20_d21_d22_tree_queries() {
    let mut db = Db::memory().unwrap();
    let scan_id = db.upsert_scan(&scan("root", ScanMode::Smart)).unwrap();
    db.insert_tree_nodes(
        scan_id,
        &[
            tree("root", None, NodeKind::Dir, 0, 0),
            tree("root/a", Some("root"), NodeKind::Dir, 1, 0),
            tree("root/a/x.txt", Some("root/a"), NodeKind::File, 2, 3),
        ],
    )
    .unwrap();

    assert_eq!(
        db.fetch_children(scan_id, "root".as_ref()).unwrap().len(),
        1
    );
    assert_eq!(
        db.fetch_subtree(scan_id, "root".as_ref(), Some(1))
            .unwrap()
            .len(),
        2
    );
    assert_eq!(
        db.fetch_subtree_filtered(scan_id, "root".as_ref(), true)
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        db.fetch_node(scan_id, "root".as_ref())
            .unwrap()
            .unwrap()
            .parent_path,
        None
    );
}

#[test]
fn d23_d24_d25_cascade_delete_scan() {
    let mut db = Db::memory().unwrap();
    let scan_id = db.upsert_scan(&scan("root", ScanMode::Smart)).unwrap();
    db.insert_dir_groups(scan_id, &[dir_group("d", 1, 1)])
        .unwrap();
    db.insert_file_groups(scan_id, &[file_group("f", false)])
        .unwrap();
    db.insert_tree_nodes(scan_id, &[tree("root", None, NodeKind::Dir, 0, 0)])
        .unwrap();
    db.delete_scan(scan_id).unwrap();

    assert!(
        db.fetch_dir_groups(scan_id, SortConfig::default())
            .unwrap()
            .is_empty()
    );
    assert!(
        db.fetch_file_groups(scan_id, SortConfig::default(), false)
            .unwrap()
            .is_empty()
    );
    assert!(db.fetch_node(scan_id, "root".as_ref()).unwrap().is_none());
}

#[test]
fn d26_d27_status_updates() {
    let mut db = Db::memory().unwrap();
    let scan_id = db.upsert_scan(&scan("root", ScanMode::Smart)).unwrap();
    db.insert_dir_groups(scan_id, &[dir_group("d", 1, 1)])
        .unwrap();
    db.insert_file_groups(scan_id, &[file_group("f", false)])
        .unwrap();
    let dir_entry = db.fetch_dir_groups(scan_id, SortConfig::default()).unwrap()[0].entries[0]
        .id
        .unwrap();
    let file_entry = db
        .fetch_file_groups(scan_id, SortConfig::default(), false)
        .unwrap()[0]
        .entries[0]
        .id
        .unwrap();

    db.update_entry_status(dir_entry, EntryStatus::Deleted)
        .unwrap();
    db.update_file_entry_status(file_entry, EntryStatus::Moved)
        .unwrap();

    assert_eq!(
        db.fetch_entry(dir_entry).unwrap().status,
        EntryStatus::Deleted
    );
    assert_eq!(
        db.fetch_file_entry(file_entry).unwrap().status,
        EntryStatus::Moved
    );
}

#[test]
fn d28_corrupt_database_errors() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("bad.db");
    fs::write(&path, b"not sqlite").unwrap();

    assert!(Db::open(&path).is_err());
}

#[test]
fn d29_unicode_round_trip_all_tables() {
    let mut db = Db::memory().unwrap();
    let scan_id = db
        .upsert_scan(&scan("日本語/café", ScanMode::Smart))
        .unwrap();
    db.insert_dir_groups(scan_id, &[dir_group("日本語", 1, 1)])
        .unwrap();
    db.insert_file_groups(scan_id, &[file_group("café", false)])
        .unwrap();
    db.insert_tree_nodes(scan_id, &[tree("日本語/café", None, NodeKind::Dir, 0, 0)])
        .unwrap();

    assert!(db.fetch_scan("日本語/café".as_ref()).unwrap().is_some());
    assert!(
        !db.fetch_dir_groups(scan_id, SortConfig::default())
            .unwrap()
            .is_empty()
    );
    assert!(
        !db.fetch_file_groups(scan_id, SortConfig::default(), false)
            .unwrap()
            .is_empty()
    );
}

#[test]
fn d30_fetch_subtree_10k_under_500ms() {
    let mut db = Db::memory().unwrap();
    let scan_id = db.upsert_scan(&scan("root", ScanMode::Smart)).unwrap();
    let mut nodes = Vec::with_capacity(10_001);
    nodes.push(tree("root", None, NodeKind::Dir, 0, 0));
    for index in 0..10_000 {
        nodes.push(tree(
            &format!("root/{index}.txt"),
            Some("root"),
            NodeKind::File,
            1,
            0,
        ));
    }
    db.insert_tree_nodes(scan_id, &nodes).unwrap();

    let start = Instant::now();
    let fetched = db.fetch_subtree(scan_id, "root".as_ref(), None).unwrap();

    assert_eq!(fetched.len(), 10_001);
    assert!(start.elapsed().as_millis() < 500);
}
