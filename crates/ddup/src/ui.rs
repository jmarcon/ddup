//! Ratatui rendering.

use std::fs;

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Clear,
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph, Wrap},
};

use crate::{
    app::{AppState, Modal, ScanStep},
    modal::modal_text,
    tree::{NodeKind, TreeNode},
};

/// Renders the application.
pub fn render(f: &mut Frame<'_>, app: &AppState) {
    f.render_widget(
        Block::new().style(Style::default().bg(DRACULA_BG).fg(DRACULA_FG)),
        f.area(),
    );
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(3),
            Constraint::Length(5),
        ])
        .split(f.area());
    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[1]);

    f.render_widget(
        Paragraph::new(format!(
            "{:?} | Root: {} | DB: {} | Sort: {:?} {:?} (s/r)",
            app.view_mode,
            app.scan_root.display(),
            app.db_path.display(),
            app.sort.by,
            app.sort.order
        ))
        .style(Style::default().bg(DRACULA_BG).fg(DRACULA_PURPLE)),
        chunks[0],
    );
    let visible_nodes = app.tree.flatten_visible();
    let list_height = usize::from(body[0].height.saturating_sub(2));
    let viewport_start = viewport_start(app.tree.cursor, list_height);
    let items = visible_nodes
        .iter()
        .skip(viewport_start)
        .take(list_height)
        .enumerate()
        .map(|(offset, node)| {
            let index = viewport_start + offset;
            let style = if index == app.tree.cursor {
                Style::default()
                    .bg(DRACULA_SELECTION)
                    .fg(DRACULA_CYAN)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().bg(DRACULA_BG).fg(DRACULA_FG)
            };
            ListItem::new(Line::from(list_label(app, node))).style(style)
        })
        .collect::<Vec<_>>();
    f.render_widget(List::new(items).block(panel("Duplicates")), body[0]);
    let details = app.tree.selected().map_or_else(
        || "No selection".to_owned(),
        |node| selected_details(app, node),
    );
    f.render_widget(
        Paragraph::new(details)
            .style(Style::default().bg(DRACULA_BG).fg(DRACULA_FG))
            .block(panel("Details")),
        body[1],
    );
    let errors = if app.scan_errors.is_empty() {
        "No scan errors".to_owned()
    } else {
        app.scan_errors
            .iter()
            .rev()
            .take(4)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    };
    f.render_widget(
        Paragraph::new(errors)
            .style(Style::default().bg(DRACULA_BG).fg(DRACULA_ORANGE))
            .block(panel("Scan errors"))
            .wrap(Wrap { trim: true }),
        chunks[2],
    );

    if app.scan_running {
        render_scan_overlay(f, app);
    }

    if !matches!(app.modal, Modal::None) {
        let area = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(20),
                Constraint::Percentage(60),
                Constraint::Percentage(20),
            ])
            .split(f.area())[1];
        let area = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(20),
                Constraint::Percentage(60),
                Constraint::Percentage(20),
            ])
            .split(area)[1];
        f.render_widget(Clear, area);
        f.render_widget(
            Paragraph::new(modal_text(&app.modal).unwrap_or_default())
                .style(Style::default().bg(DRACULA_BG).fg(DRACULA_FG))
                .block(panel("Modal")),
            area,
        );
    }
}

fn viewport_start(cursor: usize, height: usize) -> usize {
    if height == 0 {
        return 0;
    }
    cursor.saturating_sub(height.saturating_sub(1))
}

fn list_label(app: &AppState, node: &TreeNode) -> String {
    let indent = "  ".repeat(node.depth);
    let expand = if node.children.is_empty() {
        " "
    } else if node.expanded {
        "v"
    } else {
        ">"
    };
    let decision = if app.delete_selected.contains(&node.path) {
        "[D]"
    } else if app.keep_selected.contains(&node.path) {
        "[K]"
    } else {
        "   "
    };
    format!("{decision} {indent}{expand} {}", node.label)
}

fn selected_details(app: &AppState, node: &TreeNode) -> String {
    let group = node
        .group_id
        .map_or_else(|| "none".to_owned(), |value| value.to_string());
    let kind = match node.kind {
        NodeKind::GroupRoot if node.file_count.is_some() => "duplicate directory group",
        NodeKind::GroupRoot => "duplicate file group",
        NodeKind::DirEntry => "duplicate directory",
        NodeKind::FileEntry | NodeKind::FileLeaf => "duplicate file",
    };
    let file_count = node
        .file_count
        .map_or_else(|| "n/a".to_owned(), |value| value.to_string());
    let path = if node.path.as_os_str().is_empty() {
        node.label.clone()
    } else {
        node.path.display().to_string()
    };
    let relative = node
        .path
        .strip_prefix(&app.scan_root)
        .map_or_else(|_| path.clone(), |value| value.display().to_string());
    let decision = if app.delete_selected.contains(&node.path) {
        "delete"
    } else if app.keep_selected.contains(&node.path) {
        "keep"
    } else {
        "none"
    };

    let mut lines = vec![
        format!("Type: {kind}"),
        format!("Group: {group}"),
        format!("Status: {}", node.dup_marker),
        format!("Selection: {decision}"),
        format!("Entries: {}", node.entry_count),
        format!("Size: {}", format_bytes(node.size_bytes)),
        format!("Files: {file_count}"),
        format!(
            "Hash: {}",
            node.content_hash.as_deref().unwrap_or("not available")
        ),
        format!("Root: {}", app.scan_root.display()),
        format!("Relative: {relative}"),
        format!("Path: {path}"),
    ];
    lines.extend(group_copies(app, node));
    lines.extend(directory_preview(node));
    lines.join("\n")
}

fn group_copies(app: &AppState, node: &TreeNode) -> Vec<String> {
    let Some(group_id) = node.group_id else {
        return Vec::new();
    };
    let mut lines = Vec::new();
    match node.kind {
        NodeKind::GroupRoot | NodeKind::DirEntry => {
            if let Some(group) = app
                .dir_groups
                .iter()
                .find(|group| group.id == Some(group_id))
            {
                lines.push("Copies:".to_owned());
                lines.extend(
                    group
                        .entries
                        .iter()
                        .take(50)
                        .map(|entry| format!("  [{}] {}", entry.status, entry.path.display())),
                );
            }
        }
        NodeKind::FileEntry | NodeKind::FileLeaf => {
            if let Some(group) = app
                .file_groups
                .iter()
                .find(|group| group.id == Some(group_id))
            {
                lines.push("Copies:".to_owned());
                lines.extend(
                    group
                        .entries
                        .iter()
                        .take(50)
                        .map(|entry| format!("  [{}] {}", entry.status, entry.path.display())),
                );
            }
        }
    }
    lines
}

fn directory_preview(node: &TreeNode) -> Vec<String> {
    if !matches!(node.kind, NodeKind::DirEntry) || !node.path.is_dir() {
        return Vec::new();
    }
    let Ok(entries) = fs::read_dir(&node.path) else {
        return vec!["Content: unable to read directory".to_owned()];
    };
    let mut lines = vec!["Content:".to_owned()];
    for entry in entries.flatten().take(50) {
        lines.push(format!("  {}", entry.file_name().to_string_lossy()));
    }
    lines
}

fn format_bytes(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KiB", "MiB", "GiB", "TiB"];
    let mut divisor = 1_u64;
    let mut unit_index = 0;
    while unit_index + 1 < UNITS.len() {
        let Some(next_divisor) = divisor.checked_mul(1024) else {
            break;
        };
        if bytes < next_divisor {
            break;
        }
        divisor = next_divisor;
        unit_index += 1;
    }
    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        let whole = bytes / divisor;
        let fraction = bytes % divisor * 100 / divisor;
        format!("{whole}.{fraction:02} {} ({bytes} B)", UNITS[unit_index])
    }
}

fn render_scan_overlay(f: &mut Frame<'_>, app: &AppState) {
    let area = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Length(19),
            Constraint::Percentage(20),
        ])
        .split(f.area())[1];
    let area = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(50),
            Constraint::Percentage(25),
        ])
        .split(area)[1];
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8),
            Constraint::Length(6),
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Length(1),
        ])
        .split(area);
    let percent = app.step_progress.total.map_or(0, |total| {
        let value = app
            .step_progress
            .current
            .saturating_mul(100)
            .checked_div(total)
            .unwrap_or(0);
        u16::try_from(value.min(100)).unwrap_or(100)
    });
    let progress_text = app.step_progress.total.map_or_else(
        || format!("{} {}", spinner(app), app.scan_phase),
        |total| {
            format!(
                "{percent}% - {} / {} directories",
                app.step_progress.current, total
            )
        },
    );

    f.render_widget(Clear, area);
    let steps = [
        (ScanStep::Discover, "Discover filesystem"),
        (ScanStep::Hash, "Hash files and directories"),
        (ScanStep::DirGroups, "Find duplicate directories"),
        (ScanStep::FileGroups, "Find duplicate files"),
        (ScanStep::TreeStats, "Build tree statistics"),
        (ScanStep::Persist, "Save results to SQLite database"),
    ];
    let step_lines = steps
        .iter()
        .map(|(step, label)| {
            Line::from(vec![
                Span::raw(step_marker(app, *step)),
                Span::raw(format!(" {label}")),
            ])
        })
        .collect::<Vec<_>>();

    f.render_widget(
        Paragraph::new(step_lines)
            .style(Style::default().bg(DRACULA_BG).fg(DRACULA_FG))
            .block(panel("Scan progress"))
            .wrap(Wrap { trim: true }),
        rows[0],
    );
    f.render_widget(
        Paragraph::new(current_step_lines(app))
            .style(Style::default().bg(DRACULA_BG).fg(DRACULA_CYAN))
            .block(panel("Current step"))
            .wrap(Wrap { trim: true }),
        rows[1],
    );
    f.render_widget(
        Paragraph::new(progress_text)
            .alignment(Alignment::Center)
            .style(Style::default().bg(DRACULA_BG).fg(DRACULA_GREEN)),
        rows[2],
    );
    f.render_widget(
        Gauge::default()
            .block(panel("Progress"))
            .gauge_style(Style::default().fg(DRACULA_GREEN).bg(DRACULA_CURRENT_LINE))
            .percent(percent)
            .label(""),
        rows[3],
    );
    f.render_widget(
        Paragraph::new("q/Esc/Ctrl+C exits")
            .alignment(Alignment::Center)
            .style(Style::default().bg(DRACULA_BG).fg(DRACULA_COMMENT)),
        rows[4],
    );
}

const DRACULA_BG: Color = Color::Rgb(40, 42, 54);
const DRACULA_CURRENT_LINE: Color = Color::Rgb(68, 71, 90);
const DRACULA_SELECTION: Color = Color::Rgb(68, 71, 90);
const DRACULA_FG: Color = Color::Rgb(248, 248, 242);
const DRACULA_COMMENT: Color = Color::Rgb(98, 114, 164);
const DRACULA_CYAN: Color = Color::Rgb(139, 233, 253);
const DRACULA_GREEN: Color = Color::Rgb(80, 250, 123);
const DRACULA_ORANGE: Color = Color::Rgb(255, 184, 108);
const DRACULA_PURPLE: Color = Color::Rgb(189, 147, 249);

fn panel(title: &'static str) -> Block<'static> {
    Block::new()
        .title(title)
        .borders(Borders::ALL)
        .style(Style::default().bg(DRACULA_BG).fg(DRACULA_FG))
        .border_style(Style::default().fg(DRACULA_PURPLE))
}

fn step_marker(app: &AppState, step: ScanStep) -> String {
    if app.scan_done_steps.contains(&step) {
        "[x]".to_owned()
    } else if app.scan_step == step {
        format!("[{}]", spinner(app))
    } else {
        "[ ]".to_owned()
    }
}

fn spinner(app: &AppState) -> char {
    let frames = ['/', '-', '\\', '|'];
    frames[app.spinner_index % frames.len()]
}

fn shorten(value: &str, max_chars: usize) -> String {
    let mut chars = value.chars();
    let shortened = chars.by_ref().take(max_chars).collect::<String>();
    if chars.next().is_some() {
        format!("{shortened}...")
    } else {
        shortened
    }
}

fn current_step_lines(app: &AppState) -> Vec<Line<'static>> {
    vec![
        Line::from(Span::styled(
            app.scan_phase.clone(),
            Style::default()
                .fg(DRACULA_GREEN)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(app.scan_detail.clone()),
    ]
}
