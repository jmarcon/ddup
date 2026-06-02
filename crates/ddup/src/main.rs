//! ddup terminal UI.
#![allow(dead_code)]

mod app;
mod events;
mod modal;
mod tree;
mod tui_runtime;
mod ui;

use std::{path::Path, sync::mpsc, thread, time::Duration};

use anyhow::Result;
use clap::Parser;
use crossterm::event::{self, Event};
use ddup_core::{Db, ProgressTx, ScanEvent, ScanMode, ScanResult, WalkConfig, scan};

use crate::app::{AppState, Args, ViewMode};

fn main() -> Result<()> {
    let args = Args::parse();
    let mut app = AppState::new(&args)?;
    let (progress_tx, progress_rx) = mpsc::channel();
    let (result_tx, result_rx) = mpsc::channel();

    if !args.no_walk {
        app.begin_scan();
        let root = args.path.clone();
        let db_path = app.db_path.clone();
        let mode = ScanMode::from(args.mode);
        thread::spawn(move || {
            let result = scan_and_persist(&root, &db_path, mode, progress_tx);
            let _ = result_tx.send(result);
        });
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
        }
        if app.should_quit {
            break;
        }
        if event::poll(Duration::from_millis(100))?
            && let Event::Key(key) = event::read()?
        {
            events::handle_key(&mut app, key);
        }
    }
    tui_runtime::leave(&mut terminal)?;
    Ok(())
}

fn handle_scan_result(app: &mut AppState, result: Result<ScanResult>) {
    match result {
        Ok(result) => {
            apply_scan_result(app, result);
            app.finish_scan();
        }
        Err(error) => app.fail_scan(error.to_string()),
    }
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
    db.insert_file_groups(scan_id, &result.file_groups)?;
    db.insert_tree_nodes(scan_id, &result.tree_stats)?;
    Ok(())
}

fn apply_scan_result(app: &mut AppState, result: ScanResult) {
    app.current_scan = Some(result.summary);
    app.dir_groups = result.dir_groups;
    app.file_groups = result.file_groups;
    app.view_mode = ViewMode::DirsDuplicated;
    app.rebuild_tree();
}
