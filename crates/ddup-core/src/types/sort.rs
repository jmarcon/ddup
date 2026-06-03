//! Sorting types.

use serde::{Deserialize, Serialize};

/// Field used for duplicate group sorting.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SortBy {
    /// Sort by display name.
    Name,
    /// Sort by duplicate item size.
    #[default]
    TotalSize,
    /// Sort by duplicate entry count.
    FileCount,
}

/// Sort direction.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SortOrder {
    /// Ascending order.
    Asc,
    /// Descending order.
    #[default]
    Desc,
}

/// Sorting configuration.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct SortConfig {
    /// Sort field.
    pub by: SortBy,
    /// Sort direction.
    pub order: SortOrder,
}

impl SortConfig {
    /// Cycles to the next sort field.
    pub fn next_by(&mut self) {
        self.by = match self.by {
            SortBy::TotalSize => SortBy::FileCount,
            SortBy::FileCount => SortBy::Name,
            SortBy::Name => SortBy::TotalSize,
        };
    }

    /// Toggles sort direction.
    pub fn toggle_order(&mut self) {
        self.order = match self.order {
            SortOrder::Asc => SortOrder::Desc,
            SortOrder::Desc => SortOrder::Asc,
        };
    }
}
