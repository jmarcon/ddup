//! Scan metadata.

use std::{fmt, path::PathBuf, str::FromStr};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::CoreError;

/// File duplicate reporting mode.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ScanMode {
    /// Suppress duplicate files inside duplicate directories.
    #[default]
    Smart,
    /// Report all duplicate files.
    Flat,
}

impl fmt::Display for ScanMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::Smart => "smart",
            Self::Flat => "flat",
        };
        f.write_str(value)
    }
}

impl FromStr for ScanMode {
    type Err = CoreError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "smart" => Ok(Self::Smart),
            "flat" => Ok(Self::Flat),
            other => Err(CoreError::InvalidState(format!("unknown scan mode: {other}"))),
        }
    }
}

/// Persisted scan metadata.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Scan {
    /// Database id.
    pub id: Option<i64>,
    /// Scanned root path.
    pub root_path: PathBuf,
    /// Scan timestamp.
    pub scanned_at: DateTime<Utc>,
    /// Reporting mode.
    pub scan_mode: ScanMode,
    /// Number of scanned directories.
    pub total_dirs: u64,
    /// Number of scanned files.
    pub total_files: u64,
    /// Bytes that can be reclaimed.
    pub wasted_bytes: u64,
}

