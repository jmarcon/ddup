//! Application state.

use std::{fs, path::PathBuf};

use anyhow::Result;
use clap::{Parser, ValueEnum};
use ddup_core::{Db, DupFileGroup, DupGroup, Scan, ScanEvent, ScanMode, SortConfig};

use crate::tree::{TreeModel, build_tree};

/// Scan step.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScanStep {
    /// Discover filesystem.
    Discover,
    /// Hash content.
    Hash,
    /// Compute duplicate directories.
    DirGroups,
    /// Compute duplicate files.
    FileGroups,
    /// Build tree statistics.
    TreeStats,
    /// Persist results.
    Persist,
}

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
    /// Whether a scan is running.
    pub scan_running: bool,
    /// Current scan step count.
    pub scan_current: usize,
    /// Estimated scan total.
    pub scan_total: Option<usize>,
    /// Scan phase label.
    pub scan_phase: String,
    /// Current scan step.
    pub scan_step: ScanStep,
    /// Finished scan steps.
    pub scan_done_steps: Vec<ScanStep>,
    /// Spinner frame.
    pub spinner_index: usize,
    /// Current scan detail.
    pub scan_detail: String,
    /// Scan errors.
    pub scan_errors: Vec<String>,
}

impl AppState {
    /// Builds initial app state.
    pub fn new(args: &Args) -> Result<Self> {
        let db_path = args.db.clone().unwrap_or_else(default_db_path);
        if let Some(parent) = db_path.parent() {
            fs::create_dir_all(parent)?;
        }
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
            scan_running: false,
            scan_current: 0,
            scan_total: None,
            scan_phase: "Idle".to_owned(),
            scan_step: ScanStep::Discover,
            scan_done_steps: Vec::new(),
            spinner_index: 0,
            scan_detail: String::new(),
            scan_errors: Vec::new(),
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

    /// Marks scan as started.
    pub fn begin_scan(&mut self) {
        self.scan_running = true;
        self.scan_current = 0;
        self.scan_total = None;
        self.scan_step = ScanStep::Discover;
        self.scan_done_steps.clear();
        self.spinner_index = 0;
        "Discovering filesystem".clone_into(&mut self.scan_phase);
        "Counting directories and files before hashing".clone_into(&mut self.scan_detail);
        "Scanning".clone_into(&mut self.status_msg);
        self.scan_errors.clear();
    }

    /// Applies scanner progress event.
    pub fn apply_scan_event(&mut self, event: ScanEvent) {
        match event {
            ScanEvent::WalkStarted { root } => {
                "Discovering filesystem".clone_into(&mut self.scan_phase);
                self.scan_detail = format!("Walking {}", root.display());
            }
            ScanEvent::Started {
                total_dirs_estimate,
            } => {
                self.finish_step(ScanStep::Discover);
                self.scan_step = ScanStep::Hash;
                self.scan_total = Some(total_dirs_estimate);
                "Hashing files and directories".clone_into(&mut self.scan_phase);
                self.scan_detail = format!("{total_dirs_estimate} directories discovered");
            }
            ScanEvent::DirHashed { path, current } => {
                self.scan_current = current;
                "Hashing files and directories".clone_into(&mut self.scan_phase);
                self.scan_detail = format!("Current directory: {}", path.display());
            }
            ScanEvent::FileDupsComputed { count } => {
                self.finish_step(ScanStep::DirGroups);
                self.scan_step = ScanStep::FileGroups;
                "Computing duplicate file groups".clone_into(&mut self.scan_phase);
                self.scan_detail = format!("{count} duplicate file groups found");
            }
            ScanEvent::DirDupsComputed { count } => {
                self.finish_step(ScanStep::Hash);
                self.scan_step = ScanStep::DirGroups;
                "Computing duplicate directory groups".clone_into(&mut self.scan_phase);
                self.scan_detail = format!("{count} duplicate directory groups found");
            }
            ScanEvent::PersistStarted => {
                self.finish_step(ScanStep::TreeStats);
                self.scan_step = ScanStep::Persist;
                "Persisting results".clone_into(&mut self.scan_phase);
                "Writing scan, groups, file entries, and tree nodes to SQLite"
                    .clone_into(&mut self.scan_detail);
            }
            ScanEvent::TreeStatsBuilt => {
                self.finish_step(ScanStep::FileGroups);
                self.scan_step = ScanStep::TreeStats;
                "Building tree statistics".clone_into(&mut self.scan_phase);
                "Preparing treemap/sunburst data".clone_into(&mut self.scan_detail);
            }
            ScanEvent::Finished { summary } => {
                self.finish_step(ScanStep::TreeStats);
                self.scan_step = ScanStep::Persist;
                self.scan_current = summary.total_dirs.try_into().unwrap_or(usize::MAX);
                self.scan_total = Some(self.scan_current);
                "Persisting results".clone_into(&mut self.scan_phase);
                self.scan_detail = format!(
                    "{} dirs, {} files, {} wasted bytes",
                    summary.total_dirs, summary.total_files, summary.wasted_bytes
                );
            }
            ScanEvent::Error { path, message } => {
                let prefix =
                    path.map_or_else(String::new, |value| format!("{}: ", value.display()));
                self.scan_errors.push(format!("{prefix}{message}"));
            }
        }
    }

    /// Marks scan as finished.
    pub fn finish_scan(&mut self) {
        self.finish_step(ScanStep::Persist);
        self.scan_running = false;
        "Ready".clone_into(&mut self.status_msg);
        "Scan complete".clone_into(&mut self.scan_phase);
    }

    /// Records a fatal scan error.
    pub fn fail_scan(&mut self, message: String) {
        self.scan_running = false;
        self.scan_errors.push(message);
        "Scan failed".clone_into(&mut self.status_msg);
        "Scan failed".clone_into(&mut self.scan_phase);
    }

    /// Advances spinner frame.
    pub fn tick_spinner(&mut self) {
        if self.scan_running {
            self.spinner_index = self.spinner_index.wrapping_add(1);
        }
    }

    fn finish_step(&mut self, step: ScanStep) {
        if !self.scan_done_steps.contains(&step) {
            self.scan_done_steps.push(step);
        }
    }
}

fn default_db_path() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(std::env::temp_dir)
        .join("ddup")
        .join("scans.db")
}
