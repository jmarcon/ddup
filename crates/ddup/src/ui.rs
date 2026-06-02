//! Ratatui rendering.

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    text::Line,
    widgets::Clear,
    widgets::{Block, Borders, List, ListItem, Paragraph},
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
            Constraint::Length(2),
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
    f.render_widget(Paragraph::new(app.status_msg.clone()), chunks[2]);

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
