//! Walker node types.

use std::path::PathBuf;

/// File discovered by the walker.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FileEntry {
    /// File path.
    pub path: PathBuf,
    /// File size in bytes.
    pub size: u64,
}

/// Directory discovered by the walker.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DirNode {
    /// Directory path.
    pub path: PathBuf,
    /// Direct files.
    pub files: Vec<FileEntry>,
    /// Direct child directory paths.
    pub child_dirs: Vec<PathBuf>,
}

impl DirNode {
    /// Creates an empty directory node.
    #[must_use]
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            files: Vec::new(),
            child_dirs: Vec::new(),
        }
    }
}
