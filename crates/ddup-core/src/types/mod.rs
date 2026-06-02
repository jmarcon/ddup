//! Domain types.

mod hash;
mod scan;
mod status;

pub use hash::{DirHash, FileHash};
pub use scan::{Scan, ScanMode};
pub use status::{DupStatus, EntryStatus};
