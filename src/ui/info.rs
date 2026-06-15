use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::types::AppState;

pub fn render(f: &mut Frame, state: &AppState, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(5),
            Constraint::Length(8),
        ])
        .split(area);

    render_server_status(f, state, chunks[0]);
    render_notifications(f, state, chunks[1]);
    render_help(f, chunks[2]);
}

fn render_server_status(f: &mut Frame, state: &AppState, area: Rect) {
    let widget = Paragraph::new(format!(" {} users", state.online_users.len()))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Server ")
                .title_style(Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
        )
        .style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD));

    f.render_widget(widget, area);
}

fn render_notifications(f: &mut Frame, state: &AppState, area: Rect) {
    let items: Vec<ListItem> = state
        .notifications
        .iter()
        .rev()
        .take(10)
        .rev()
        .map(|n| ListItem::new(Line::from(Span::styled(n, Style::default().fg(Color::Cyan)))))
        .collect();

    let widget = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Activity ")
            .title_style(Style::default().fg(Color::Yellow)),
    );

    f.render_widget(widget, area);
}

fn render_help(f: &mut Frame, area: Rect) {
    let text = Text::from(vec![
        Line::from(Span::styled("Controls:", Style::default().add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from("Up/Down: Scroll chat"),
        Line::from("Enter: Send message"),
        Line::from("Backspace: Delete"),
        Line::from("c (empty input): Exit"),
    ]);

    let widget = Paragraph::new(text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Help ")
                .title_style(Style::default().fg(Color::Gray)),
        )
        .style(Style::default().fg(Color::DarkGray));

    f.render_widget(widget, area);
}
