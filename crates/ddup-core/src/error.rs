//! Error types for ddup-core.

use std::{io, path::PathBuf};

/// Core library error.
#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    /// I/O error tied to a path.
    #[error("I/O error at {path}: {source}")]
    Io {
        /// Path involved in the operation.
        path: PathBuf,
        /// Source I/O error.
        source: io::Error,
    },
    /// Database error.
    #[error("database error: {0}")]
    Db(#[from] rusqlite::Error),
    /// Path does not exist.
    #[error("path not found: {0}")]
    PathNotFound(PathBuf),
    /// Invalid internal state.
    #[error("invalid state: {0}")]
    InvalidState(String),
    /// Filesystem walk error.
    #[error("walk error: {0}")]
    Walk(String),
}

/// Core result type.
pub type Result<T> = std::result::Result<T, CoreError>;
