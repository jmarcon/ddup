#![allow(missing_docs, clippy::expect_used, clippy::panic, clippy::unwrap_used)]

use std::fs;

use chrono::Utc;
use ddup_core::{
    CoreError, Db, DirHash, DupEntry, DupFileEntry, DupFileGroup, DupGroup, EntryStatus, FileHash,
    Scan, ScanMode, SortConfig, delete_entry, delete_file_entry, move_entry, move_file_entry,
};
use tempfile::TempDir;

fn scan(root: &str) -> Scan {
    Scan {
        id: None,
        root_path: root.into(),
        scanned_at: Utc::now(),
        scan_mode: ScanMode::Smart,
        total_dirs: 1,
        total_files: 1,
        wasted_bytes: 1,
    }
}

fn setup_dir_entry() -> (TempDir, Db, i64) {
    let dir = TempDir::new().unwrap();
    let victim = dir.path().join("victim");
    fs::create_dir_all(&victim).unwrap();
    fs::write(victim.join("x.txt"), b"x").unwrap();
    let mut db = Db::memory().unwrap();
    let scan_id = db.upsert_scan(&scan("root")).unwrap();
    db.insert_dir_groups(
        scan_id,
        &[DupGroup {
            id: None,
            dir_hash: DirHash::new("h"),
            file_count: 1,
            size_bytes: 1,
            entries: vec![
                DupEntry {
                    id: None,
                    path: victim,
                    status: EntryStatus::Pending,
                    dir_hash: DirHash::new("h"),
                },
                DupEntry {
                    id: None,
                    path: dir.path().join("other"),
                    status: EntryStatus::Pending,
                    dir_hash: DirHash::new("h"),
                },
            ],
        }],
    )
    .unwrap();
    let entry_id = db.fetch_dir_groups(scan_id, SortConfig::default()).unwrap()[0].entries[1]
        .id
        .unwrap();
    (dir, db, entry_id)
}

fn setup_file_entry() -> (TempDir, Db, i64) {
    let dir = TempDir::new().unwrap();
    let victim = dir.path().join("victim.txt");
    fs::write(&victim, b"x").unwrap();
    let mut db = Db::memory().unwrap();
    let scan_id = db.upsert_scan(&scan("root")).unwrap();
    db.insert_file_groups(
        scan_id,
        &[DupFileGroup {
            id: None,
            file_hash: FileHash::new("h"),
            size_bytes: 1,
            entries: vec![
                DupFileEntry {
                    id: None,
                    path: dir.path().join("other.txt"),
                    status: EntryStatus::Pending,
                    suppressed_by_dir_group: None,
                },
                DupFileEntry {
                    id: None,
                    path: victim,
                    status: EntryStatus::Pending,
                    suppressed_by_dir_group: None,
                },
            ],
        }],
    )
    .unwrap();
    let entry_id = db
        .fetch_file_groups(scan_id, SortConfig::default(), false)
        .unwrap()[0]
        .entries[1]
        .id
        .unwrap();
    (dir, db, entry_id)
}

#[test]
fn a1_delete_dir_and_status() {
    let (dir, db, entry_id) = setup_dir_entry();
    let victim = dir.path().join("victim");

    delete_entry(&db, entry_id).unwrap();

    assert!(!victim.exists());
    assert_eq!(
        db.fetch_entry(entry_id).unwrap().status,
        EntryStatus::Deleted
    );
}

#[test]
fn a2_delete_missing_dir_errors() {
    let (dir, db, entry_id) = setup_dir_entry();
    fs::remove_dir_all(dir.path().join("victim")).unwrap();

    assert!(matches!(
        delete_entry(&db, entry_id),
        Err(CoreError::PathNotFound(_))
    ));
}

#[test]
fn a3_move_dir() {
    let (dir, db, entry_id) = setup_dir_entry();
    let dest = dir.path().join("moved");

    move_entry(&db, entry_id, &dest).unwrap();

    assert!(dest.exists());
    assert_eq!(db.fetch_entry(entry_id).unwrap().status, EntryStatus::Moved);
}

#[test]
fn a4_move_dest_exists_errors() {
    let (dir, db, entry_id) = setup_dir_entry();
    let dest = dir.path().join("dest");
    fs::create_dir_all(&dest).unwrap();

    assert!(matches!(
        move_entry(&db, entry_id, &dest),
        Err(CoreError::InvalidState(_))
    ));
}

#[test]
fn a5_dir_db_consistency_after_move() {
    let (dir, db, entry_id) = setup_dir_entry();
    move_entry(&db, entry_id, &dir.path().join("moved")).unwrap();

    assert_eq!(db.fetch_entry(entry_id).unwrap().status, EntryStatus::Moved);
}

#[test]
fn a6_delete_file() {
    let (dir, db, entry_id) = setup_file_entry();
    let victim = dir.path().join("victim.txt");

    delete_file_entry(&db, entry_id).unwrap();

    assert!(!victim.exists());
    assert_eq!(
        db.fetch_file_entry(entry_id).unwrap().status,
        EntryStatus::Deleted
    );
}

#[test]
fn a7_move_file() {
    let (dir, db, entry_id) = setup_file_entry();
    let dest = dir.path().join("moved.txt");

    move_file_entry(&db, entry_id, &dest).unwrap();

    assert!(dest.exists());
    assert_eq!(
        db.fetch_file_entry(entry_id).unwrap().status,
        EntryStatus::Moved
    );
}

#[test]
fn a8_file_db_consistency_after_delete() {
    let (_dir, db, entry_id) = setup_file_entry();
    delete_file_entry(&db, entry_id).unwrap();

    assert_eq!(
        db.fetch_file_entry(entry_id).unwrap().status,
        EntryStatus::Deleted
    );
}
