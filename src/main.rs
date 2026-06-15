mod constants;
mod crypto;
mod handlers;
mod input;
mod types;
pub mod ui;

use std::{io, sync::Arc, time::Duration};

use crossterm::{
    event::{self, Event, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use futures_util::StreamExt;
use ratatui::{backend::CrosstermBackend, Terminal};
use tokio::sync::Mutex;
use tokio_tungstenite::{connect_async, tungstenite::Message};

use constants::WS_URL;
use types::AppState;

/// Convenience alias for the shared WebSocket write half.
type WsSink = futures_util::stream::SplitSink<
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
    Message,
>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Switch to TUI alternate screen
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout))?;

    let state = Arc::new(Mutex::new(AppState::new()));

    ui::show_loading_screen(&mut terminal, &state).await;

    // Connect and split WebSocket into independent read/write halves
    let (ws_stream, _) = connect_async(WS_URL).await?;
    let (write, mut read) = ws_stream.split();
    let write = Arc::new(Mutex::new(write));

    state.lock().await.is_connected = true;

    // Background task: receive and dispatch incoming frames
    let ws_task = {
        let state = state.clone();
        let write = write.clone();
        tokio::spawn(async move {
            while let Some(Ok(Message::Text(text))) = read.next().await {
                handlers::handle_ws_message(&text, &state, &write).await;
            }
        })
    };

    let result = run_app(&mut terminal, state, write).await;

    // Restore terminal before exiting
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    ws_task.abort();

    if let Err(err) = result {
        eprintln!("Error: {:?}", err);
    }

    Ok(())
}

/// Main event loop: draw -> poll input -> repeat until exit.
async fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    state: Arc<Mutex<AppState>>,
    write: Arc<Mutex<WsSink>>,
) -> io::Result<()> {
    loop {
        {
            let s = state.lock().await;
            terminal.draw(|f| ui::ui(f, &s))?;
        }

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    if input::handle_key_event(key.code, &state, &write).await {
                        break;
                    }
                }
            }
        }
    }

    Ok(())
}
