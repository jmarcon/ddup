//! ddup terminal UI.
#![allow(dead_code)]

mod app;
mod events;
mod modal;
mod tree;
mod tui_runtime;
mod ui;

use std::{
    io::{self, Write},
    path::Path,
    sync::mpsc,
    thread,
    time::Duration,
};

use anyhow::Result;
use clap::Parser;
use crossterm::event::{self, Event, KeyEventKind};
use ddup_core::{
    Db, DupFileGroup, ProgressTx, ScanEvent, ScanMode, ScanResult, SortConfig, TreeStats,
    WalkConfig, scan,
};

use crate::app::{AppState, Args, ViewMode};

fn main() -> Result<()> {
    let args = Args::parse();
    let mut app = AppState::new(&args)?;
    let (progress_tx, progress_rx) = mpsc::channel();
    let (result_tx, result_rx) = mpsc::channel();

    if args.no_tui {
        let root = args.path.clone();
        let db_path = app.db_path.clone();
        let mode = ScanMode::from(args.mode);
        let result = scan_and_persist(&root, &db_path, mode, progress_tx)?;
        let mut stdout = io::stdout().lock();
        writeln!(stdout, "SQLite: {}", db_path.display())?;
        writeln!(
            stdout,
            "Scan complete: {} dirs, {} files, {} wasted bytes",
            result.summary.total_dirs, result.summary.total_files, result.summary.wasted_bytes
        )?;
        return Ok(());
    }

    let mut scan_started = false;
    if !args.rescan {
        let root = args.path.clone();
        if load_scan_from_db(&mut app, &root)? {
            scan_started = true;
        }
    }

    if !args.no_walk && (!scan_started || args.rescan) {
        start_scan(
            &mut app,
            args.path.clone(),
            ScanMode::from(args.mode),
            progress_tx.clone(),
            result_tx.clone(),
        );
    }

    let mut terminal = tui_runtime::enter()?;
    loop {
        terminal.draw(|frame| ui::render(frame, &app))?;
        for event in progress_rx.try_iter() {
            app.apply_scan_event(event);
        }
        app.tick_spinner();
        if let Ok(result) = result_rx.try_recv() {
            handle_scan_result(&mut app, result);
            if args.exit_after_scan {
                app.should_quit = true;
            }
        }
        if app.should_quit {
            break;
        }
        if event::poll(Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    events::handle_key(&mut app, key);
                }
                Event::Mouse(mouse) => {
                    let size = terminal.size()?;
                    events::handle_mouse(&mut app, mouse, size.width, size.height);
                }
                _ => {}
            }
        }
        if app.rescan_requested && !app.scan_running {
            app.rescan_requested = false;
            start_scan(
                &mut app,
                args.path.clone(),
                ScanMode::from(args.mode),
                progress_tx.clone(),
                result_tx.clone(),
            );
        }
    }
    tui_runtime::leave(&mut terminal)?;
    Ok(())
}

fn handle_scan_result(app: &mut AppState, result: Result<ScanResult>) {
    match result {
        Ok(result) => {
            if let Err(error) = apply_scan_result(app, result) {
                app.fail_scan(error.to_string());
            } else {
                app.finish_scan();
            }
        }
        Err(error) => app.fail_scan(error.to_string()),
    }
}

fn start_scan(
    app: &mut AppState,
    root: std::path::PathBuf,
    mode: ScanMode,
    progress_tx: ProgressTx,
    result_tx: mpsc::Sender<Result<ScanResult>>,
) {
    app.scan_root.clone_from(&root);
    app.begin_scan();
    let db_path = app.db_path.clone();
    thread::spawn(move || {
        let result = scan_and_persist(&root, &db_path, mode, progress_tx);
        let _ = result_tx.send(result);
    });
}

fn scan_and_persist(
    root: &Path,
    db_path: &Path,
    mode: ScanMode,
    progress_tx: ProgressTx,
) -> Result<ScanResult> {
    let result = scan(
        root,
        &WalkConfig::default(),
        mode,
        Some(progress_tx.clone()),
    )?;
    let _ = progress_tx.send(ScanEvent::PersistStarted);
    persist_scan_result(db_path, &result)?;
    Ok(result)
}

fn persist_scan_result(db_path: &Path, result: &ScanResult) -> Result<()> {
    let mut db = Db::open(db_path)?;
    let scan_id = db.upsert_scan(&result.summary)?;
    db.insert_dir_groups(scan_id, &result.dir_groups)?;
    let file_groups = remap_file_suppression_group_ids(&db, scan_id, result)?;
    db.insert_file_groups(scan_id, &file_groups)?;
    let tree_stats = remap_tree_group_ids(&db, scan_id, result)?;
    db.insert_tree_nodes(scan_id, &tree_stats)?;
    Ok(())
}

fn remap_file_suppression_group_ids(
    db: &Db,
    scan_id: i64,
    result: &ScanResult,
) -> Result<Vec<DupFileGroup>> {
    let db_dir_groups = db.fetch_dir_groups(scan_id, SortConfig::default())?;
    let mut file_groups = result.file_groups.clone();

    for group in &mut file_groups {
        for entry in &mut group.entries {
            entry.suppressed_by_dir_group = entry.suppressed_by_dir_group.and_then(|group_id| {
                result
                    .dir_groups
                    .iter()
                    .find(|group| group.id == Some(group_id))
                    .and_then(|group| {
                        db_dir_groups
                            .iter()
                            .find(|db_group| db_group.dir_hash == group.dir_hash)
                            .and_then(|db_group| db_group.id)
                    })
            });
        }
    }

    Ok(file_groups)
}

fn remap_tree_group_ids(db: &Db, scan_id: i64, result: &ScanResult) -> Result<Vec<TreeStats>> {
    let db_dir_groups = db.fetch_dir_groups(scan_id, SortConfig::default())?;
    let db_file_groups = db.fetch_file_groups(scan_id, SortConfig::default(), false)?;
    let mut tree_stats = result.tree_stats.clone();

    for node in &mut tree_stats {
        node.dir_group_id = node.dir_group_id.and_then(|group_id| {
            result
                .dir_groups
                .iter()
                .find(|group| group.id == Some(group_id))
                .and_then(|group| {
                    db_dir_groups
                        .iter()
                        .find(|db_group| db_group.dir_hash == group.dir_hash)
                        .and_then(|db_group| db_group.id)
                })
        });
        node.file_group_id = node.file_group_id.and_then(|group_id| {
            result
                .file_groups
                .iter()
                .find(|group| group.id == Some(group_id))
                .and_then(|group| {
                    db_file_groups
                        .iter()
                        .find(|db_group| db_group.file_hash == group.file_hash)
                        .and_then(|db_group| db_group.id)
                })
        });
    }

    Ok(tree_stats)
}

fn load_scan_from_db(app: &mut AppState, root: &Path) -> Result<bool> {
    let Some(scan) = app.db.fetch_scan(root)? else {
        return Ok(false);
    };
    let Some(scan_id) = scan.id else {
        return Ok(false);
    };
    app.current_scan = Some(scan);
    app.dir_groups = app.db.fetch_dir_groups(scan_id, app.sort)?;
    app.file_groups = app.db.fetch_file_groups(scan_id, app.sort, false)?;
    app.scan_root = root.to_path_buf();
    app.view_mode = ViewMode::DirsDuplicated;
    app.rebuild_tree();
    Ok(true)
}

fn apply_scan_result(app: &mut AppState, result: ScanResult) -> Result<()> {
    let root = result.summary.root_path;
    let _ = load_scan_from_db(app, &root)?;
    Ok(())
}
