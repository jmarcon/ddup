//! Domain types.

mod dup;
mod hash;
mod scan;
mod status;
mod tree;

pub use dup::{DupEntry, DupFileEntry, DupFileGroup, DupGroup};
pub use hash::{DirHash, FileHash};
pub use scan::{Scan, ScanMode};
pub use status::{DupStatus, EntryStatus};
pub use tree::{NodeKind, TreeStats};
