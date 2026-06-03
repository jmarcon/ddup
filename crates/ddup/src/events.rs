//! Keyboard event handling.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};

use crate::app::{AppState, Modal, ViewMode};

/// Handles one key event.
pub fn handle_key(app: &mut AppState, key: KeyEvent) {
    if !matches!(app.modal, Modal::None) {
        if matches!(key.code, KeyCode::Esc) {
            app.modal = Modal::None;
        }
        return;
    }

    match key.code {
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.should_quit = true;
        }
        KeyCode::Char('r') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.rescan_requested = true;
        }
        KeyCode::F(5) => app.rescan_requested = true,
        KeyCode::Char('q') | KeyCode::Esc => app.should_quit = true,
        KeyCode::Char('j') | KeyCode::Down => app.tree.move_cursor(1),
        KeyCode::Char('k') | KeyCode::Up => app.tree.move_cursor(-1),
        KeyCode::Char('h') | KeyCode::Left => app.tree.close_selected(),
        KeyCode::Char('l') | KeyCode::Right => app.tree.open_selected(),
        KeyCode::Enter => app.tree.toggle_expand(),
        KeyCode::Char('d') => app.toggle_delete_selected(),
        KeyCode::Char('m') => app.toggle_keep_selected(),
        KeyCode::Char('u') => app.clear_selected_decision(),
        KeyCode::Char('s') => {
            app.sort.next_by();
            app.rebuild_tree();
        }
        KeyCode::Char('o' | 'r') => {
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
}

/// Handles one mouse event.
pub fn handle_mouse(
    app: &mut AppState,
    mouse: MouseEvent,
    terminal_width: u16,
    terminal_height: u16,
) {
    match mouse.kind {
        MouseEventKind::ScrollDown => app.tree.move_cursor(3),
        MouseEventKind::ScrollUp => app.tree.move_cursor(-3),
        MouseEventKind::Down(_) => {
            if mouse.column >= terminal_width / 2 {
                return;
            }
            let body_height = terminal_height.saturating_sub(6);
            let list_height = usize::from(body_height.saturating_sub(2));
            if mouse.row < 2 || mouse.row >= 2 + u16::try_from(list_height).unwrap_or(u16::MAX) {
                return;
            }
            let start = mouse_viewport_start(app.tree.cursor, list_height);
            let offset = usize::from(mouse.row.saturating_sub(2));
            app.tree.select(start + offset);
        }
        _ => {}
    }
}

fn mouse_viewport_start(cursor: usize, height: usize) -> usize {
    if height == 0 {
        return 0;
    }
    cursor.saturating_sub(height.saturating_sub(1))
}
