//! Ratatui rendering.

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout},
    text::Line,
    widgets::Clear,
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph, Wrap},
};

use crate::{
    app::{AppState, Modal},
    modal::modal_text,
};

/// Renders the application.
pub fn render(f: &mut Frame<'_>, app: &AppState) {
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

    f.render_widget(Paragraph::new(format!("{:?}", app.view_mode)), chunks[0]);
    let items = app
        .tree
        .flatten_visible()
        .iter()
        .map(|node| ListItem::new(Line::from(node.label.clone())))
        .collect::<Vec<_>>();
    f.render_widget(
        List::new(items).block(Block::new().borders(Borders::ALL)),
        body[0],
    );
    let details = app.tree.selected().map_or_else(
        || "No selection".to_owned(),
        |node| node.path.display().to_string(),
    );
    f.render_widget(
        Paragraph::new(details).block(Block::new().borders(Borders::ALL)),
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
            .block(Block::new().title("Scan errors").borders(Borders::ALL))
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
                .block(Block::new().borders(Borders::ALL)),
            area,
        );
    }
}

fn render_scan_overlay(f: &mut Frame<'_>, app: &AppState) {
    let area = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(30),
            Constraint::Length(11),
            Constraint::Percentage(30),
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
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(1),
        ])
        .split(area);
    let percent = app.scan_total.map_or(0, |total| {
        let value = app
            .scan_current
            .saturating_mul(100)
            .checked_div(total)
            .unwrap_or(0);
        u16::try_from(value.min(100)).unwrap_or(100)
    });
    let label = app.scan_total.map_or_else(
        || "Discovery phase".to_owned(),
        |total| format!("{percent}% - {} / {} directories", app.scan_current, total),
    );

    f.render_widget(Clear, area);
    f.render_widget(
        Paragraph::new(app.scan_phase.clone())
            .alignment(Alignment::Center)
            .block(Block::new().title("Scan progress").borders(Borders::ALL))
            .wrap(Wrap { trim: true }),
        rows[0],
    );
    f.render_widget(
        Paragraph::new(shorten(&app.scan_detail, 120))
            .alignment(Alignment::Center)
            .block(Block::new().title("Current step").borders(Borders::ALL))
            .wrap(Wrap { trim: true }),
        rows[1],
    );
    f.render_widget(
        Gauge::default()
            .block(Block::new().title("Progress").borders(Borders::ALL))
            .gauge_style(ratatui::style::Style::default())
            .percent(percent)
            .label(label),
        rows[2],
    );
    f.render_widget(
        Paragraph::new("q/Esc exits").alignment(Alignment::Center),
        rows[3],
    );
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
