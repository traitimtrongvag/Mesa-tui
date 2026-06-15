use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::types::AppState;

pub fn render(f: &mut Frame, state: &AppState, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(3)])
        .split(area);

    render_messages(f, state, chunks[0]);
    render_input(f, state, chunks[1]);
}

fn render_messages(f: &mut Frame, state: &AppState, area: Rect) {
    let items: Vec<ListItem> = state
        .messages
        .iter()
        .rev()
        .skip(state.scroll_offset)
        .take(100)
        .rev()
        .map(|msg| {
            let content_style = if msg.is_own {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::Yellow)
            };
            let prefix = if msg.is_own {
                format!("[{}] {} (you): ", msg.timestamp, msg.sender)
            } else {
                format!("[{}] {}: ", msg.timestamp, msg.sender)
            };
            ListItem::new(Line::from(vec![
                Span::styled(prefix, Style::default().fg(Color::Gray)),
                Span::styled(&msg.content, content_style),
            ]))
        })
        .collect();

    let scroll_indicator = if state.scroll_offset > 0 {
        format!(" ^{} ", state.scroll_offset)
    } else {
        String::new()
    };

    let widget = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" Chat{} ", scroll_indicator))
            .title_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
    );

    f.render_widget(widget, area);
}

fn render_input(f: &mut Frame, state: &AppState, area: Rect) {
    let display = if !state.paste_buffer.is_empty() {
        format!("{}[pasting...]", state.input)
    } else {
        state.input.clone()
    };

    let widget = Paragraph::new(display.as_str())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" [{}] ", state.token))
                .title_style(Style::default().fg(Color::Green)),
        )
        .style(Style::default().fg(Color::White))
        .wrap(Wrap { trim: false });

    f.render_widget(widget, area);

    // Place terminal cursor at cursor_position; clamp to inner border.
    let cursor_x = (area.x + 1 + state.cursor_position as u16).min(area.x + area.width - 2);
    let cursor_y = area.y + 1;
    f.set_cursor_position((cursor_x, cursor_y));
}
