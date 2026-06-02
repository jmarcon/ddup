//! Walker configuration.

/// Configuration for filesystem traversal.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WalkConfig {
    /// Whether symlink targets are followed.
    pub follow_symlinks: bool,
    /// Whether hidden entries are ignored.
    pub ignore_hidden: bool,
    /// Path component names ignored by default.
    pub ignore_patterns: Vec<String>,
    /// Minimum file size included in results.
    pub min_file_size: u64,
}

impl Default for WalkConfig {
    fn default() -> Self {
        Self {
            follow_symlinks: false,
            ignore_hidden: false,
            ignore_patterns: vec![
                "node_modules".to_owned(),
                ".git".to_owned(),
                "target".to_owned(),
            ],
            min_file_size: 0,
        }
    }
}

impl WalkConfig {
    /// Sets symlink-following behavior.
    #[must_use]
    pub fn with_follow_symlinks(mut self, value: bool) -> Self {
        self.follow_symlinks = value;
        self
    }

    /// Sets hidden-entry filtering.
    #[must_use]
    pub fn with_ignore_hidden(mut self, value: bool) -> Self {
        self.ignore_hidden = value;
        self
    }

    /// Replaces ignored path component names.
    #[must_use]
    pub fn with_ignore_patterns(
        mut self,
        values: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        self.ignore_patterns = values.into_iter().map(Into::into).collect();
        self
    }

    /// Sets minimum file size.
    #[must_use]
    pub fn with_min_file_size(mut self, value: u64) -> Self {
        self.min_file_size = value;
        self
    }
}
