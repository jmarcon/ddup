//! ddup terminal UI.
#![allow(dead_code)]

mod app;
mod events;
mod modal;
mod tree;
mod tui_runtime;
mod ui;

use std::time::Duration;

use anyhow::Result;
use clap::Parser;
use crossterm::event::{self, Event};
use ddup_core::{ScanMode, WalkConfig, scan};

use crate::app::{AppState, Args, ViewMode};

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let args = Args::parse();
    let mut app = AppState::new(&args)?;

    if !args.no_walk {
        "Scanning".clone_into(&mut app.status_msg);
        let result = scan(
            &args.path,
            &WalkConfig::default(),
            ScanMode::from(args.mode),
            None,
        )?;
        let scan_id = app.db.upsert_scan(&result.summary)?;
        app.db.insert_dir_groups(scan_id, &result.dir_groups)?;
        app.db.insert_file_groups(scan_id, &result.file_groups)?;
        app.db.insert_tree_nodes(scan_id, &result.tree_stats)?;
        app.current_scan = Some(result.summary);
        app.dir_groups = result.dir_groups;
        app.file_groups = result.file_groups;
        app.view_mode = ViewMode::DirsDuplicated;
        "Ready".clone_into(&mut app.status_msg);
        app.rebuild_tree();
    }

    let mut terminal = tui_runtime::enter()?;
    loop {
        terminal.draw(|frame| ui::render(frame, &app))?;
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
