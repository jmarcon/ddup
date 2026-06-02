//! ddup-core: engine de detecção de duplicatas (diretórios e arquivos).

pub mod actions;
pub mod db;
pub mod error;
pub mod hasher;
pub mod progress;
pub mod scanner;
pub mod types;
pub mod walker;

pub use actions::{delete_entry, delete_file_entry, move_entry, move_file_entry, open_in_explorer};
pub use db::Db;
pub use error::{CoreError, Result};
pub use hasher::{hash_dir, hash_file};
pub use progress::{ProgressTx, ScanEvent};
pub use scanner::{ScanResult, scan};
pub use types::{
    DirHash, DupEntry, DupFileEntry, DupFileGroup, DupGroup, DupStatus, EntryStatus, FileHash,
    NodeKind, Scan, ScanMode, SortBy, SortConfig, SortOrder, TreeStats,
};
pub use walker::{DirNode, FileEntry, WalkConfig, walk, walk_with_errors};
