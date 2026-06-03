//! Keyboard event handling.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};

use std::path::PathBuf;

use crate::app::{AppState, Modal, ViewMode};

/// Handles one key event.
pub fn handle_key(app: &mut AppState, key: KeyEvent) {
    if !matches!(app.modal, Modal::None) {
        handle_modal_key(app, key);
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
        KeyCode::Char('d') => app.confirm_delete_selected(),
        KeyCode::Char('m') => app.prompt_move_selected(),
        KeyCode::Char('o') => app.open_selected(),
        KeyCode::Char('c') => app.copy_selected_path(),
        KeyCode::Char('x') => app.toggle_delete_selected(),
        KeyCode::Char('p') => app.toggle_keep_selected(),
        KeyCode::Char('u') => app.clear_selected_decision(),
        KeyCode::Char('a') => app.apply_marked_actions(),
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
}

fn handle_modal_key(app: &mut AppState, key: KeyEvent) {
    match app.modal.clone() {
        Modal::Help | Modal::Error { .. } => {
            if matches!(key.code, KeyCode::Esc | KeyCode::Enter) {
                app.modal = Modal::None;
            }
        }
        Modal::ConfirmDelete { entry_id, file } => match key.code {
            KeyCode::Char('y' | 'Y') => app.confirm_delete(entry_id, file),
            KeyCode::Char('n' | 'N') | KeyCode::Esc => app.modal = Modal::None,
            _ => {}
        },
        Modal::MovePrompt {
            entry_id,
            file,
            mut input,
        } => match key.code {
            KeyCode::Enter => app.confirm_move(entry_id, file, &PathBuf::from(input)),
            KeyCode::Esc => app.modal = Modal::None,
            KeyCode::Backspace => {
                input.pop();
                app.modal = Modal::MovePrompt {
                    entry_id,
                    file,
                    input,
                };
            }
            KeyCode::Char(value) => {
                input.push(value);
                app.modal = Modal::MovePrompt {
                    entry_id,
                    file,
                    input,
                };
            }
            _ => {}
        },
        Modal::None => {}
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

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::unwrap_used)]

    use std::path::PathBuf;

    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    use super::*;
    use crate::{
        app::{Args, CliScanMode},
        tree::{NodeKind, TreeModel, TreeNode},
    };

    fn app() -> AppState {
        let base = std::env::temp_dir().join(format!("ddup-events-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&base);
        let args = Args {
            path: base.clone(),
            db: Some(base.join("events.sqlite")),
            mode: CliScanMode::Smart,
            rescan: false,
            no_walk: true,
            exit_after_scan: false,
            no_tui: true,
        };
        let mut app = AppState::new(&args).unwrap();
        app.tree = TreeModel {
            cursor: 0,
            roots: vec![
                TreeNode {
                    path: PathBuf::from("a"),
                    entry_id: Some(1),
                    label: "a".to_owned(),
                    depth: 0,
                    kind: NodeKind::DirEntry,
                    expanded: false,
                    children: Vec::new(),
                    group_id: Some(1),
                    dup_marker: "pending".to_owned(),
                    entry_count: 1,
                    size_bytes: 1,
                    file_count: Some(1),
                    content_hash: Some("hash".to_owned()),
                },
                TreeNode {
                    path: PathBuf::from("b"),
                    entry_id: Some(2),
                    label: "b".to_owned(),
                    depth: 0,
                    kind: NodeKind::DirEntry,
                    expanded: false,
                    children: Vec::new(),
                    group_id: Some(1),
                    dup_marker: "pending".to_owned(),
                    entry_count: 1,
                    size_bytes: 1,
                    file_count: Some(1),
                    content_hash: Some("hash".to_owned()),
                },
            ],
        };
        app
    }

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::empty())
    }

    #[test]
    fn down_moves_one_item() {
        let mut app = app();

        handle_key(&mut app, key(KeyCode::Down));

        assert_eq!(app.tree.cursor, 1);
    }

    #[test]
    fn esc_closes_help_without_quitting() {
        let mut app = app();
        app.modal = Modal::Help;

        handle_key(&mut app, key(KeyCode::Esc));

        assert_eq!(app.modal, Modal::None);
        assert!(!app.should_quit);
    }

    #[test]
    fn d_opens_delete_confirmation() {
        let mut app = app();

        handle_key(&mut app, key(KeyCode::Char('d')));

        assert_eq!(
            app.modal,
            Modal::ConfirmDelete {
                entry_id: 1,
                file: false
            }
        );
    }

    #[test]
    fn marks_delete_keep_and_clear() {
        let mut app = app();

        handle_key(&mut app, key(KeyCode::Char('x')));
        assert!(app.delete_selected.contains(&PathBuf::from("a")));

        handle_key(&mut app, key(KeyCode::Char('p')));
        assert!(!app.delete_selected.contains(&PathBuf::from("a")));
        assert!(app.keep_selected.contains(&PathBuf::from("a")));

        handle_key(&mut app, key(KeyCode::Char('u')));
        assert!(app.keep_selected.is_empty());
    }
}
