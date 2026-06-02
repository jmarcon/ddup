//! Status enums.

use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};

use crate::CoreError;

/// Mutable status for an entry selected by the user.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum EntryStatus {
    /// Entry has not been acted on.
    Pending,
    /// Entry was intentionally kept.
    Kept,
    /// Entry was deleted.
    Deleted,
    /// Entry was moved.
    Moved,
}

impl fmt::Display for EntryStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::Pending => "pending",
            Self::Kept => "kept",
            Self::Deleted => "deleted",
            Self::Moved => "moved",
        };
        f.write_str(value)
    }
}

impl FromStr for EntryStatus {
    type Err = CoreError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "pending" => Ok(Self::Pending),
            "kept" => Ok(Self::Kept),
            "deleted" => Ok(Self::Deleted),
            "moved" => Ok(Self::Moved),
            other => Err(CoreError::InvalidState(format!(
                "unknown entry status: {other}"
            ))),
        }
    }
}

/// Duplicate status for a tree node.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DupStatus {
    /// Node is unique.
    Unique,
    /// Node is a duplicated directory.
    DupDir,
    /// Node is a duplicated file.
    DupFile,
    /// Node contains duplicated descendants.
    Partial,
}

impl fmt::Display for DupStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::Unique => "unique",
            Self::DupDir => "dup_dir",
            Self::DupFile => "dup_file",
            Self::Partial => "partial",
        };
        f.write_str(value)
    }
}

impl FromStr for DupStatus {
    type Err = CoreError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "unique" => Ok(Self::Unique),
            "dup_dir" => Ok(Self::DupDir),
            "dup_file" => Ok(Self::DupFile),
            "partial" => Ok(Self::Partial),
            other => Err(CoreError::InvalidState(format!(
                "unknown duplicate status: {other}"
            ))),
        }
    }
}
