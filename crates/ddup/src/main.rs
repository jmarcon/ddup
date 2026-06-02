//! ddup terminal UI.
#![allow(dead_code)]

mod app;
mod events;
mod modal;
mod tree;
mod tui_runtime;
mod ui;

use std::{sync::mpsc, thread, time::Duration};

use anyhow::Result;
use clap::Parser;
use crossterm::event::{self, Event};
use ddup_core::{ScanMode, ScanResult, WalkConfig, scan};

use crate::app::{AppState, Args, ViewMode};

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let args = Args::parse();
    let mut app = AppState::new(&args)?;
    let (progress_tx, progress_rx) = mpsc::channel();
    let (result_tx, result_rx) = mpsc::channel();

    if !args.no_walk {
        app.begin_scan();
        let root = args.path.clone();
        let mode = ScanMode::from(args.mode);
        thread::spawn(move || {
            let result = scan(&root, &WalkConfig::default(), mode, Some(progress_tx));
            let _ = result_tx.send(result);
        });
    }

    let mut terminal = tui_runtime::enter()?;
    loop {
        terminal.draw(|frame| ui::render(frame, &app))?;
        for event in progress_rx.try_iter() {
            app.apply_scan_event(event);
        }
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

fn handle_scan_result(app: &mut AppState, result: ddup_core::Result<ScanResult>) {
    match result {
        Ok(result) => {
            if let Err(error) = persist_scan_result(app, result) {
                app.fail_scan(error.to_string());
            } else {
                app.finish_scan();
            }
        }
        Err(error) => app.fail_scan(error.to_string()),
    }
}

fn persist_scan_result(app: &mut AppState, result: ScanResult) -> Result<()> {
    let scan_id = app.db.upsert_scan(&result.summary)?;
    app.db.insert_dir_groups(scan_id, &result.dir_groups)?;
    app.db.insert_file_groups(scan_id, &result.file_groups)?;
    app.db.insert_tree_nodes(scan_id, &result.tree_stats)?;
    app.current_scan = Some(result.summary);
    app.dir_groups = result.dir_groups;
    app.file_groups = result.file_groups;
    app.view_mode = ViewMode::DirsDuplicated;
    app.rebuild_tree();
    Ok(())
}
