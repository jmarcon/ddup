//! Keyboard event handling.

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};

use crate::app::{AppState, Modal, ViewMode};

/// Handles one key event.
pub fn handle_key(app: &mut AppState, key: KeyEvent) -> Result<()> {
    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => app.should_quit = true,
        KeyCode::Char('j') | KeyCode::Down => app.tree.move_cursor(1),
        KeyCode::Char('k') | KeyCode::Up => app.tree.move_cursor(-1),
        KeyCode::Char('h')
        | KeyCode::Left
        | KeyCode::Char('l')
        | KeyCode::Right
        | KeyCode::Enter => {
            app.tree.toggle_expand();
        }
        KeyCode::Char('s') => {
            app.sort.next_by();
            app.rebuild_tree();
        }
        KeyCode::Char('r') => {
            app.sort.toggle_order();
            app.rebuild_tree();
        }
        KeyCode::Char('f') => {
            app.view_mode = match app.view_mode {
                ViewMode::DirsDuplicated => ViewMode::FilesDuplicatedSmart,
                ViewMode::FilesDuplicatedSmart => ViewMode::FilesDuplicatedFlat,
                ViewMode::FilesDuplicatedFlat => ViewMode::DirsDuplicated,
            };
            app.status_msg = format!("{:?}", app.view_mode);
            app.rebuild_tree();
        }
        KeyCode::Char('?') => app.modal = Modal::Help,
        _ => {}
    }
    Ok(())
}
