use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

use crate::types::AppState;

pub fn render(f: &mut Frame, state: &AppState, area: Rect) {
    let items: Vec<ListItem> = state
        .online_users
        .iter()
        .map(|user| {
            let is_me = user == &state.token;
            let prefix = if is_me { "> " } else { "  " };
            let style = if is_me {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default().fg(Color::White)
            };
            ListItem::new(Line::from(Span::styled(format!("{}{}", prefix, user), style)))
        })
        .collect();

    let widget = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Online Users ")
            .title_style(Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD)),
    );

    f.render_widget(widget, area);
}
