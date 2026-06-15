pub mod chat;
pub mod info;
pub mod users;

use std::{sync::Arc, time::{Duration, Instant}};

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::Paragraph,
    Frame, Terminal,
};
use tokio::sync::Mutex;

use crate::{constants::LOADING_ANIMATION_SPEED_MS, types::AppState};

/// Root layout: users panel | chat area | info panel.
pub fn ui(f: &mut Frame, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(20),
            Constraint::Percentage(75),
            Constraint::Percentage(25),
        ])
        .split(f.area());

    users::render(f, state, chunks[0]);
    chat::render(f, state, chunks[1]);
    info::render(f, state, chunks[2]);
}

/// Animated "Connecting..." screen shown before the WebSocket handshake completes.
/// Breaks after connection or a 30-second timeout.
pub async fn show_loading_screen<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    state: &Arc<Mutex<AppState>>,
) {
    let mut dot_count = 1usize;
    let start = Instant::now();

    loop {
        if state.lock().await.is_connected || start.elapsed().as_secs() > 30 {
            break;
        }

        let _ = terminal.draw(|f| {
            let area = f.area();
            let text = format!("Connecting to server{:<3}", ".".repeat(dot_count));
            let len = text.len() as u16;
            let widget = Paragraph::new(text)
                .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));
            let x = (area.width.saturating_sub(len)) / 2;
            let y = area.height / 2;
            f.render_widget(widget, Rect::new(x, y, len, 1));
        });

        tokio::time::sleep(Duration::from_millis(LOADING_ANIMATION_SPEED_MS)).await;
        dot_count = (dot_count % 3) + 1;
    }
}
