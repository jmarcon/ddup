//! Application state.

use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, ValueEnum};
use ddup_core::{Db, DupFileGroup, DupGroup, Scan, ScanMode, SortConfig};

use crate::tree::{TreeModel, build_tree};

/// CLI arguments.
#[derive(Clone, Debug, Parser)]
pub struct Args {
    /// Path to scan.
    pub path: PathBuf,
    /// SQLite database path.
    #[arg(long)]
    pub db: Option<PathBuf>,
    /// Scan mode.
    #[arg(long, value_enum, default_value_t = CliScanMode::Smart)]
    pub mode: CliScanMode,
    /// Force rescan.
    #[arg(long)]
    pub rescan: bool,
    /// Skip scanning.
    #[arg(long)]
    pub no_walk: bool,
}

/// CLI scan mode.
#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum CliScanMode {
    /// Smart mode.
    Smart,
    /// Flat mode.
    Flat,
}

impl From<CliScanMode> for ScanMode {
    fn from(value: CliScanMode) -> Self {
        match value {
            CliScanMode::Smart => Self::Smart,
            CliScanMode::Flat => Self::Flat,
        }
    }
}

/// Current visible view.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ViewMode {
    /// Duplicate directories.
    DirsDuplicated,
    /// Duplicate files with Smart suppression.
    FilesDuplicatedSmart,
    /// Duplicate files without suppression.
    FilesDuplicatedFlat,
}

/// Focus target.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Focus {
    /// Tree pane.
    Tree,
    /// Details pane.
    Details,
    /// Modal.
    Modal,
}

/// Modal state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Modal {
    /// No modal.
    None,
    /// Delete confirmation.
    ConfirmDelete { entry_id: i64, file: bool },
    /// Move prompt.
    MovePrompt {
        entry_id: i64,
        file: bool,
        input: String,
    },
    /// Help modal.
    Help,
    /// Error modal.
    Error { message: String },
}

/// Mutable app state.
pub struct AppState {
    /// Database.
    pub db: Db,
    /// Current scan.
    pub current_scan: Option<Scan>,
    /// Current view mode.
    pub view_mode: ViewMode,
    /// Directory groups.
    pub dir_groups: Vec<DupGroup>,
    /// File groups.
    pub file_groups: Vec<DupFileGroup>,
    /// Sort config.
    pub sort: SortConfig,
    /// Focus.
    pub focus: Focus,
    /// Modal.
    pub modal: Modal,
    /// Status line.
    pub status_msg: String,
    /// Quit flag.
    pub should_quit: bool,
    /// Visible tree.
    pub tree: TreeModel,
}

impl AppState {
    /// Builds initial app state.
    pub fn new(args: &Args) -> Result<Self> {
        let db_path = args.db.clone().unwrap_or_else(default_db_path);
        let db = Db::open(&db_path)?;
        let sort = SortConfig::default();
        Ok(Self {
            db,
            current_scan: None,
            view_mode: ViewMode::DirsDuplicated,
            dir_groups: Vec::new(),
            file_groups: Vec::new(),
            sort,
            focus: Focus::Tree,
            modal: Modal::None,
            status_msg: "Ready".to_owned(),
            should_quit: false,
            tree: build_tree(ViewMode::DirsDuplicated, &[], &[], sort),
        })
    }

    /// Rebuilds visible tree.
    pub fn rebuild_tree(&mut self) {
        self.tree = build_tree(
            self.view_mode,
            &self.dir_groups,
            &self.file_groups,
            self.sort,
        );
    }
}

fn default_db_path() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(std::env::temp_dir)
        .join("ddup")
        .join("scans.db")
}
