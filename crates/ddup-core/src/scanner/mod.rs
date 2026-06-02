//! Scan pipeline.

mod dir_dups;
mod file_dups;
mod pipeline;
mod tree_stats;

pub use pipeline::{ScanResult, scan};
