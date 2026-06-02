//! Scan progress events.

use std::{path::PathBuf, sync::mpsc};

use crate::Scan;

/// Progress event emitted by the scanner.
#[derive(Clone, Debug)]
pub enum ScanEvent {
    /// Filesystem discovery started.
    WalkStarted {
        /// Scan root.
        root: PathBuf,
    },
    /// Scan started.
    Started {
        /// Estimated directory count.
        total_dirs_estimate: usize,
    },
    /// Directory was hashed.
    DirHashed {
        /// Directory path.
        path: PathBuf,
        /// Current hashed directory count.
        current: usize,
    },
    /// File duplicate groups were computed.
    FileDupsComputed {
        /// Duplicate group count.
        count: usize,
    },
    /// Directory duplicate groups were computed.
    DirDupsComputed {
        /// Duplicate group count.
        count: usize,
    },
    /// Result persistence started.
    PersistStarted,
    /// Tree statistics were built.
    TreeStatsBuilt,
    /// Scan finished.
    Finished {
        /// Scan summary.
        summary: Scan,
    },
    /// Non-fatal scan error.
    Error {
        /// Optional path.
        path: Option<PathBuf>,
        /// Error message.
        message: String,
    },
}

/// Progress channel sender.
pub type ProgressTx = mpsc::Sender<ScanEvent>;

/// Sends an event if a progress channel exists.
pub fn send_event(progress: Option<&ProgressTx>, event: ScanEvent) {
    if let Some(tx) = progress {
        let _ = tx.send(event);
    }
}
