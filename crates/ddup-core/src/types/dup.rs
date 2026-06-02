//! Duplicate group models.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::{DirHash, EntryStatus, FileHash};

/// Directory duplicate entry.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct DupEntry {
    /// Database id.
    pub id: Option<i64>,
    /// Directory path.
    pub path: PathBuf,
    /// User action status.
    pub status: EntryStatus,
    /// Directory hash.
    pub dir_hash: DirHash,
}

/// Directory duplicate group.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct DupGroup {
    /// Database id.
    pub id: Option<i64>,
    /// Shared directory hash.
    pub dir_hash: DirHash,
    /// Recursive file count.
    pub file_count: u64,
    /// Recursive size.
    pub size_bytes: u64,
    /// Duplicate directory entries.
    pub entries: Vec<DupEntry>,
}

/// File duplicate entry.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct DupFileEntry {
    /// Database id.
    pub id: Option<i64>,
    /// File path.
    pub path: PathBuf,
    /// User action status.
    pub status: EntryStatus,
    /// Directory group that suppresses this file in Smart mode.
    pub suppressed_by_dir_group: Option<i64>,
}

/// File duplicate group.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct DupFileGroup {
    /// Database id.
    pub id: Option<i64>,
    /// Shared file hash.
    pub file_hash: FileHash,
    /// Per-file size.
    pub size_bytes: u64,
    /// Duplicate file entries.
    pub entries: Vec<DupFileEntry>,
}

