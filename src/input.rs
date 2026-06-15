use std::{sync::Arc, time::Instant};

use chrono::Local;
use crossterm::event::KeyCode;
use tokio::sync::Mutex;

use crate::{
    crypto::{initiate_key_exchange, send_encrypted_message},
    types::{AppState, ChatMsg},
    WsSink,
};

/// Handles a single key press. Returns `true` when the app should exit.
pub async fn handle_key_event(
    key_code: KeyCode,
    state: &Arc<Mutex<AppState>>,
    write: &Arc<Mutex<WsSink>>,
) -> bool {
    let mut s = state.lock().await;

    match key_code {
        // Exit only when the input box is empty (prevents accidental quit mid-type)
        KeyCode::Char('c') if s.input.is_empty() => return true,

        KeyCode::Char(c) => {
            let pos = s.cursor_position;
            s.input.insert(pos, c);
            s.cursor_position += 1;
            s.last_input_time = Instant::now();
        }
        KeyCode::Backspace => {
            if s.cursor_position > 0 {
                s.cursor_position -= 1;
                let pos = s.cursor_position;
                s.input.remove(pos);
            }
        }
        KeyCode::Delete => {
            if s.cursor_position < s.input.len() {
                let pos = s.cursor_position;
                s.input.remove(pos);
            }
        }
        KeyCode::Left => {
            if s.cursor_position > 0 {
                s.cursor_position -= 1;
            }
        }
        KeyCode::Right => {
            if s.cursor_position < s.input.len() {
                s.cursor_position += 1;
            }
        }
        KeyCode::Home => s.cursor_position = 0,
        KeyCode::End => s.cursor_position = s.input.len(),
        KeyCode::Up => s.scroll_offset += 1,
        KeyCode::Down => {
            if s.scroll_offset > 0 {
                s.scroll_offset -= 1;
            }
        }
        KeyCode::Enter => {
            handle_enter_key(&mut s, write).await;
        }
        _ => {}
    }

    false
}

/// Sends the current input to every peer, performing key exchange if needed,
/// then appends the message locally and resets the input box.
async fn handle_enter_key(state: &mut AppState, write: &Arc<Mutex<WsSink>>) {
    if state.input.trim().is_empty() {
        return;
    }

    let message = state.input.clone();
    let my_token = state.token.clone();
    let recipients = state.online_users.clone();

    for recipient in recipients {
        if recipient == my_token {
            continue;
        }

        if !state.aes_keys.contains_key(&recipient) {
            let _ = initiate_key_exchange(&recipient, &my_token, state, write).await;
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }

        if state.aes_keys.contains_key(&recipient) {
            let _ = send_encrypted_message(&recipient, &message, state, write).await;
        }
    }

    let timestamp = Local::now().format("%H:%M").to_string();
    state.messages.push(ChatMsg {
        content: message,
        is_own: true,
        timestamp,
        sender: my_token,
    });

    state.input.clear();
    state.cursor_position = 0;
    state.scroll_offset = 0;
}
