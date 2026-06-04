//! Application state.

use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

use anyhow::Result;
use clap::{Parser, ValueEnum};
use ddup_core::{
    Db, DupFileGroup, DupGroup, EntryStatus, Scan, ScanEvent, ScanMode, SortConfig, delete_entry,
    delete_file_entry, move_entry, move_file_entry, open_in_explorer,
};

use crate::tree::{NodeKind, TreeModel, build_tree};

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

/// Step progress.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StepProgress {
    /// Current amount.
    pub current: usize,
    /// Total amount if known.
    pub total: Option<usize>,
}

/// CLI arguments.
#[derive(Clone, Debug, Parser)]
#[allow(clippy::struct_excessive_bools)]
pub struct Args {
    /// Paths to scan. With 2+ paths, duplicates are compared between roots.
    #[arg(required = true, num_args = 1..)]
    pub paths: Vec<PathBuf>,
    /// SQLite database path.
    #[arg(long)]
    pub db: Option<PathBuf>,
    /// Use an in-memory SQLite database instead of a database file.
    #[arg(long, visible_alias = "in-memory-db", conflicts_with = "db")]
    pub memory_db: bool,
    /// Scan mode.
    #[arg(long, value_enum, default_value_t = CliScanMode::Smart)]
    pub mode: CliScanMode,
    /// Force rescan.
    #[arg(long)]
    pub rescan: bool,
    /// Skip scanning.
    #[arg(long)]
    pub no_walk: bool,
    /// Exit TUI after scan finishes.
    #[arg(long)]
    pub exit_after_scan: bool,
    /// Run scan without opening the TUI.
    #[arg(long)]
    pub no_tui: bool,
    /// Disable Nerd Font file and folder icons.
    #[arg(long)]
    pub no_icons: bool,
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
#[allow(clippy::struct_excessive_bools)]
pub struct AppState {
    /// Database.
    pub db: Db,
    /// Database path.
    pub db_path: PathBuf,
    /// Whether database is in-memory.
    pub db_in_memory: bool,
    /// Scan root path.
    pub scan_root: PathBuf,
    /// Scan root paths.
    pub scan_roots: Vec<PathBuf>,
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
    /// Paths selected to delete.
    pub delete_selected: HashSet<PathBuf>,
    /// Paths selected to keep.
    pub keep_selected: HashSet<PathBuf>,
    /// Whether a scan is running.
    pub scan_running: bool,
    /// Current step progress.
    pub step_progress: StepProgress,
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
    /// Re-scan requested by UI.
    pub rescan_requested: bool,
    /// Whether Nerd Font icons are shown.
    pub icons_enabled: bool,
}

impl AppState {
    /// Builds initial app state.
    pub fn new(args: &Args) -> Result<Self> {
        let (db_path, db) = if args.memory_db {
            (PathBuf::from(":memory:"), Db::memory()?)
        } else {
            let db_path = args.db.clone().unwrap_or_else(default_db_path);
            if let Some(parent) = db_path.parent() {
                fs::create_dir_all(parent)?;
            }
            let db = Db::open(&db_path)?;
            (db_path, db)
        };
        let sort = SortConfig::default();
        Ok(Self {
            db,
            db_path,
            db_in_memory: args.memory_db,
            scan_root: scan_root_label(&args.paths),
            scan_roots: args.paths.clone(),
            current_scan: None,
            view_mode: ViewMode::DirsDuplicated,
            dir_groups: Vec::new(),
            file_groups: Vec::new(),
            sort,
            focus: Focus::Tree,
            modal: Modal::None,
            status_msg: "Ready".to_owned(),
            should_quit: false,
            tree: build_tree(ViewMode::DirsDuplicated, &[], &[], sort, &args.paths),
            delete_selected: HashSet::new(),
            keep_selected: HashSet::new(),
            scan_running: false,
            step_progress: StepProgress {
                current: 0,
                total: None,
            },
            scan_phase: "Idle".to_owned(),
            scan_step: ScanStep::Discover,
            scan_done_steps: Vec::new(),
            spinner_index: 0,
            scan_detail: String::new(),
            scan_errors: Vec::new(),
            rescan_requested: false,
            icons_enabled: !args.no_icons,
        })
    }

    /// Toggles Nerd Font icons.
    pub fn toggle_icons(&mut self) {
        self.icons_enabled = !self.icons_enabled;
        self.status_msg = if self.icons_enabled {
            "Icons enabled".to_owned()
        } else {
            "Icons disabled".to_owned()
        };
    }

    /// Rebuilds visible tree.
    pub fn rebuild_tree(&mut self) {
        self.tree = build_tree(
            self.view_mode,
            &self.dir_groups,
            &self.file_groups,
            self.sort,
            &self.scan_roots,
        );
    }

    /// Toggles selected path as delete target.
    pub fn toggle_delete_selected(&mut self) {
        let paths = self.selected_mark_paths();
        if paths.is_empty() {
            return;
        }
        let all_marked = paths.iter().all(|path| self.delete_selected.contains(path));
        for path in &paths {
            self.keep_selected.remove(path);
            if all_marked {
                self.delete_selected.remove(path);
            } else {
                self.delete_selected.insert(path.clone());
            }
        }
        if all_marked {
            self.status_msg = format!("Delete mark cleared for {} item(s)", paths.len());
        } else {
            self.status_msg = format!("Marked {} item(s) for delete", paths.len());
        }
    }

    /// Toggles selected path as keep target.
    pub fn toggle_keep_selected(&mut self) {
        let paths = self.selected_mark_paths();
        if paths.is_empty() {
            return;
        }
        let all_marked = paths.iter().all(|path| self.keep_selected.contains(path));
        for path in &paths {
            self.delete_selected.remove(path);
            if all_marked {
                self.keep_selected.remove(path);
            } else {
                self.keep_selected.insert(path.clone());
            }
        }
        if all_marked {
            self.status_msg = format!("Keep mark cleared for {} item(s)", paths.len());
        } else {
            self.status_msg = format!("Marked {} item(s) to keep", paths.len());
        }
    }

    /// Clears selected path decision.
    pub fn clear_selected_decision(&mut self) {
        let paths = self.selected_mark_paths();
        if paths.is_empty() {
            return;
        }
        for path in &paths {
            self.delete_selected.remove(path);
            self.keep_selected.remove(path);
        }
        self.status_msg = format!("Cleared marks for {} item(s)", paths.len());
    }

    /// Opens a delete confirmation for the selected entry.
    pub fn confirm_delete_selected(&mut self) {
        let Some((entry_id, file)) = self.selected_action_target() else {
            "Select a file or directory entry first".clone_into(&mut self.status_msg);
            return;
        };
        self.modal = Modal::ConfirmDelete { entry_id, file };
    }

    /// Opens a move prompt for the selected entry.
    pub fn prompt_move_selected(&mut self) {
        let Some((entry_id, file)) = self.selected_action_target() else {
            "Select a file or directory entry first".clone_into(&mut self.status_msg);
            return;
        };
        self.modal = Modal::MovePrompt {
            entry_id,
            file,
            input: String::new(),
        };
    }

    /// Opens selected entry in the platform file manager.
    pub fn open_selected(&mut self) {
        let Some(path) = self.tree.selected().map(|node| node.path.clone()) else {
            return;
        };
        if path.as_os_str().is_empty() {
            "Select a file or directory entry first".clone_into(&mut self.status_msg);
            return;
        }
        match open_in_explorer(&path) {
            Ok(()) => self.status_msg = format!("Opened {}", path.display()),
            Err(error) => {
                self.modal = Modal::Error {
                    message: error.to_string(),
                }
            }
        }
    }

    /// Copies selected entry path to the clipboard.
    pub fn copy_selected_path(&mut self) {
        let Some(path) = self.tree.selected().map(|node| node.path.clone()) else {
            return;
        };
        if path.as_os_str().is_empty() {
            "Select a file or directory entry first".clone_into(&mut self.status_msg);
            return;
        }
        let text = path.display().to_string();
        match arboard::Clipboard::new().and_then(|mut clipboard| clipboard.set_text(text.clone())) {
            Ok(()) => self.status_msg = format!("Copied {text}"),
            Err(error) => {
                self.modal = Modal::Error {
                    message: format!("Could not copy path: {error}"),
                };
            }
        }
    }

    /// Confirms modal delete action.
    pub fn confirm_delete(&mut self, entry_id: i64, file: bool) {
        let result = if file {
            delete_file_entry(&self.db, entry_id)
        } else {
            delete_entry(&self.db, entry_id)
        };
        self.after_action(result, "Deleted");
    }

    /// Confirms modal move action.
    pub fn confirm_move(&mut self, entry_id: i64, file: bool, dest: &Path) {
        let result = if file {
            move_file_entry(&self.db, entry_id, dest)
        } else {
            move_entry(&self.db, entry_id, dest)
        };
        self.after_action(result, "Moved");
    }

    /// Applies all delete/keep marks.
    pub fn apply_marked_actions(&mut self) {
        let mut changed = 0_usize;
        for group in self.dir_groups.clone() {
            for entry in group.entries {
                if self.delete_selected.contains(&entry.path) {
                    if let Some(id) = entry.id {
                        if let Err(error) = delete_entry(&self.db, id) {
                            self.modal = Modal::Error {
                                message: error.to_string(),
                            };
                            return;
                        }
                        changed += 1;
                    }
                } else if self.keep_selected.contains(&entry.path)
                    && let Some(id) = entry.id
                {
                    if let Err(error) = self.db.update_entry_status(id, EntryStatus::Kept) {
                        self.modal = Modal::Error {
                            message: error.to_string(),
                        };
                        return;
                    }
                    changed += 1;
                }
            }
        }
        for group in self.file_groups.clone() {
            for entry in group.entries {
                if self.delete_selected.contains(&entry.path) {
                    if let Some(id) = entry.id {
                        if let Err(error) = delete_file_entry(&self.db, id) {
                            self.modal = Modal::Error {
                                message: error.to_string(),
                            };
                            return;
                        }
                        changed += 1;
                    }
                } else if self.keep_selected.contains(&entry.path)
                    && let Some(id) = entry.id
                {
                    if let Err(error) = self.db.update_file_entry_status(id, EntryStatus::Kept) {
                        self.modal = Modal::Error {
                            message: error.to_string(),
                        };
                        return;
                    }
                    changed += 1;
                }
            }
        }
        self.delete_selected.clear();
        self.keep_selected.clear();
        self.reload_current_scan();
        self.status_msg = format!("Applied {changed} marked actions");
    }

    /// Marks scan as started.
    pub fn begin_scan(&mut self) {
        self.scan_running = true;
        self.step_progress = StepProgress {
            current: 0,
            total: None,
        };
        self.scan_step = ScanStep::Discover;
        self.scan_done_steps.clear();
        self.spinner_index = 0;
        "Discovering filesystem".clone_into(&mut self.scan_phase);
        "Counting directories and files before hashing".clone_into(&mut self.scan_detail);
        "Scanning".clone_into(&mut self.status_msg);
        self.scan_errors.clear();
        self.delete_selected.clear();
        self.keep_selected.clear();
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
                self.step_progress = StepProgress {
                    current: 0,
                    total: Some(total_dirs_estimate),
                };
                "Hashing files and directories".clone_into(&mut self.scan_phase);
                self.scan_detail = format!("{total_dirs_estimate} directories discovered");
            }
            ScanEvent::DirHashed { path, current } => {
                self.step_progress.current = current;
                "Hashing files and directories".clone_into(&mut self.scan_phase);
                self.scan_detail = format!("Current directory: {}", path.display());
            }
            ScanEvent::FileDupsComputed { count } => {
                self.finish_step(ScanStep::DirGroups);
                self.scan_step = ScanStep::FileGroups;
                self.step_progress = StepProgress {
                    current: 0,
                    total: None,
                };
                "Computing duplicate file groups".clone_into(&mut self.scan_phase);
                self.scan_detail = format!("{count} duplicate file groups found");
            }
            ScanEvent::DirDupsComputed { count } => {
                self.finish_step(ScanStep::Hash);
                self.scan_step = ScanStep::DirGroups;
                self.step_progress = StepProgress {
                    current: 0,
                    total: None,
                };
                "Computing duplicate directory groups".clone_into(&mut self.scan_phase);
                self.scan_detail = format!("{count} duplicate directory groups found");
            }
            ScanEvent::PersistStarted => {
                self.finish_step(ScanStep::TreeStats);
                self.scan_step = ScanStep::Persist;
                self.step_progress = StepProgress {
                    current: 0,
                    total: None,
                };
                "Persisting results".clone_into(&mut self.scan_phase);
                "Writing scan, groups, file entries, and tree nodes to SQLite"
                    .clone_into(&mut self.scan_detail);
            }
            ScanEvent::TreeStatsBuilt => {
                self.finish_step(ScanStep::FileGroups);
                self.scan_step = ScanStep::TreeStats;
                self.step_progress = StepProgress {
                    current: 0,
                    total: None,
                };
                "Building tree statistics".clone_into(&mut self.scan_phase);
                "Preparing treemap/sunburst data".clone_into(&mut self.scan_detail);
            }
            ScanEvent::Finished { summary } => {
                self.finish_step(ScanStep::TreeStats);
                self.scan_step = ScanStep::Persist;
                self.step_progress = StepProgress {
                    current: 0,
                    total: None,
                };
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

    fn selected_action_target(&self) -> Option<(i64, bool)> {
        let node = self.tree.selected()?;
        let entry_id = node.entry_id?;
        match node.kind {
            NodeKind::DirEntry => Some((entry_id, false)),
            NodeKind::FileEntry | NodeKind::FileLeaf => Some((entry_id, true)),
            NodeKind::GroupRoot => None,
        }
    }

    fn selected_mark_paths(&self) -> Vec<PathBuf> {
        let Some(node) = self.tree.selected() else {
            return Vec::new();
        };
        if !node.path.as_os_str().is_empty() {
            return vec![node.path.clone()];
        }
        let Some(group_id) = node.group_id else {
            return Vec::new();
        };
        match self.view_mode {
            ViewMode::DirsDuplicated => self
                .dir_groups
                .iter()
                .find(|group| group.id == Some(group_id))
                .map(|group| {
                    group
                        .entries
                        .iter()
                        .map(|entry| entry.path.clone())
                        .collect()
                })
                .unwrap_or_default(),
            ViewMode::FilesDuplicatedSmart | ViewMode::FilesDuplicatedFlat => self
                .file_groups
                .iter()
                .find(|group| group.id == Some(group_id))
                .map(|group| {
                    group
                        .entries
                        .iter()
                        .map(|entry| entry.path.clone())
                        .collect()
                })
                .unwrap_or_default(),
        }
    }

    fn after_action(&mut self, result: ddup_core::Result<()>, label: &str) {
        match result {
            Ok(()) => {
                self.modal = Modal::None;
                self.reload_current_scan();
                label.clone_into(&mut self.status_msg);
            }
            Err(error) => {
                self.modal = Modal::Error {
                    message: error.to_string(),
                };
            }
        }
    }

    fn reload_current_scan(&mut self) {
        let Some(scan_id) = self.current_scan.as_ref().and_then(|scan| scan.id) else {
            self.rebuild_tree();
            return;
        };
        match (
            self.db.fetch_dir_groups(scan_id, self.sort),
            self.db.fetch_file_groups(
                scan_id,
                self.sort,
                self.view_mode != ViewMode::FilesDuplicatedFlat,
            ),
        ) {
            (Ok(dir_groups), Ok(file_groups)) => {
                self.dir_groups = dir_groups;
                self.file_groups = file_groups;
                self.rebuild_tree();
            }
            (Err(error), _) | (_, Err(error)) => {
                self.modal = Modal::Error {
                    message: error.to_string(),
                };
            }
        }
    }
}

fn default_db_path() -> PathBuf {
    if let Some(path) = std::env::var_os("DDUP_DATA_LOCAL_DIR") {
        return PathBuf::from(path).join("ddup").join("scans.db");
    }
    dirs::data_local_dir()
        .unwrap_or_else(std::env::temp_dir)
        .join("ddup")
        .join("scans.db")
}

/// Builds stable scan label used as SQLite key for one or more roots.
#[must_use]
pub fn scan_root_label(roots: &[PathBuf]) -> PathBuf {
    if roots.len() == 1 {
        return roots[0].clone();
    }
    PathBuf::from(
        roots
            .iter()
            .map(|root| root.display().to_string())
            .collect::<Vec<_>>()
            .join(" | "),
    )
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    #[test]
    fn memory_db_flag_uses_in_memory_database() {
        let args = Args::try_parse_from(["ddup", "root", "--memory-db"]).unwrap();
        let app = AppState::new(&args).unwrap();

        assert!(app.db_in_memory);
        assert_eq!(app.db_path, PathBuf::from(":memory:"));
    }

    #[test]
    fn memory_db_conflicts_with_db_path() {
        let result = Args::try_parse_from(["ddup", "root", "--memory-db", "--db", "ddup.sqlite"]);

        assert!(result.is_err());
    }
}
