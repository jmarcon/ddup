#![allow(missing_docs, clippy::expect_used, clippy::panic, clippy::unwrap_used)]

use std::{fs, path::Path};

use ddup_core::{CoreError, DirHash, FileHash, hash_dir, hash_file};
use pretty_assertions::assert_eq;
use tempfile::TempDir;

fn write_file(dir: &TempDir, name: &str, bytes: &[u8]) -> std::path::PathBuf {
    let path = dir.path().join(name);
    fs::write(&path, bytes).unwrap();
    path
}

#[test]
fn h1_empty_file_hash() {
    let dir = TempDir::new().unwrap();
    let path = write_file(&dir, "empty", b"");

    assert_eq!(
        hash_file(&path).unwrap().as_str(),
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
}

#[test]
fn h2_hello_world_hash() {
    let dir = TempDir::new().unwrap();
    let path = write_file(&dir, "hello", b"hello world");

    assert_eq!(
        hash_file(&path).unwrap().as_str(),
        "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
    );
}

#[test]
fn h3_changing_one_byte_changes_hash() {
    let dir = TempDir::new().unwrap();
    let a = write_file(&dir, "a", b"hello world");
    let b = write_file(&dir, "b", b"hello worle");

    assert_ne!(hash_file(&a).unwrap(), hash_file(&b).unwrap());
}

#[test]
fn h10_missing_path_is_io_error() {
    let result = hash_file(Path::new("missing-file"));

    assert!(matches!(result, Err(CoreError::Io { .. })));
}

#[test]
fn h4_same_hashes_same_dir_hash() {
    let files = vec![FileHash::new("a"), FileHash::new("b")];

    assert_eq!(hash_dir(&files, &[]), hash_dir(&files, &[]));
}

#[test]
fn h5_input_order_does_not_matter() {
    let left = vec![FileHash::new("b"), FileHash::new("a")];
    let right = vec![FileHash::new("a"), FileHash::new("b")];

    assert_eq!(hash_dir(&left, &[]), hash_dir(&right, &[]));
}

#[test]
fn h6_changing_hash_changes_dir_hash() {
    assert_ne!(
        hash_dir(&[FileHash::new("a")], &[]),
        hash_dir(&[FileHash::new("b")], &[])
    );
}

#[test]
fn h7_adding_hash_changes_dir_hash() {
    assert_ne!(
        hash_dir(&[FileHash::new("a")], &[]),
        hash_dir(&[FileHash::new("a"), FileHash::new("b")], &[])
    );
}

#[test]
fn h8_removing_hash_changes_dir_hash() {
    assert_ne!(
        hash_dir(&[FileHash::new("a"), FileHash::new("b")], &[]),
        hash_dir(&[FileHash::new("a")], &[])
    );
}

#[test]
fn h9_child_dir_hashes_only_are_defined() {
    let hash = hash_dir(&[], &[DirHash::new("child")]);

    assert!(!hash.as_str().is_empty());
}

#[test]
fn h11_file_and_dir_hash_inputs_are_typed() {
    assert_ne!(
        hash_dir(&[FileHash::new("same")], &[]),
        hash_dir(&[], &[DirHash::new("same")])
    );
}
