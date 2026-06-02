//! Tree statistics models.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::DupStatus;

/// Tree node kind.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum NodeKind {
    /// Directory node.
    Dir,
    /// File node.
    File,
}

/// Precomputed tree statistics for GUI visualizations.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct TreeStats {
    /// Node path.
    pub path: PathBuf,
    /// Node kind.
    pub kind: NodeKind,
    /// Parent node path.
    pub parent_path: Option<PathBuf>,
    /// Depth from scan root.
    pub depth: u32,
    /// Own size.
    pub size_bytes: u64,
    /// Recursive size.
    pub size_recursive: u64,
    /// Recursive file count.
    pub file_count_recursive: u64,
    /// Lowercase file extension without dot.
    pub extension: Option<String>,
    /// Duplicate status.
    pub dup_status: DupStatus,
    /// Duplicate directory group id.
    pub dir_group_id: Option<i64>,
    /// Duplicate file group id.
    pub file_group_id: Option<i64>,
    /// Reclaimable bytes.
    pub wasted_bytes: u64,
}
