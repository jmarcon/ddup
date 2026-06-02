//! ddup-core: engine de detecção de duplicatas (diretórios e arquivos).

pub mod error;
pub mod hasher;
pub mod types;

pub use error::{CoreError, Result};
pub use hasher::{hash_dir, hash_file};
pub use types::{
    DirHash, DupEntry, DupFileEntry, DupFileGroup, DupGroup, DupStatus, EntryStatus, FileHash,
    NodeKind, Scan, ScanMode, SortBy, SortConfig, SortOrder, TreeStats,
};
