//! ddup-core: engine de detecção de duplicatas (diretórios e arquivos).

pub mod db;
pub mod error;
pub mod hasher;
pub mod progress;
pub mod scanner;
pub mod types;
pub mod walker;

pub use db::Db;
pub use error::{CoreError, Result};
pub use hasher::{hash_dir, hash_file};
pub use progress::{ProgressTx, ScanEvent};
pub use scanner::{ScanResult, scan};
pub use types::{
    DirHash, DupEntry, DupFileEntry, DupFileGroup, DupGroup, DupStatus, EntryStatus, FileHash,
    NodeKind, Scan, ScanMode, SortBy, SortConfig, SortOrder, TreeStats,
};
pub use walker::{DirNode, FileEntry, WalkConfig, walk};
