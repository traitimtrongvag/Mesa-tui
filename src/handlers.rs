use std::sync::Arc;

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Key, Nonce,
};
use base64::{engine::general_purpose, Engine as _};
use chrono::Local;
use rsa::{
    pkcs8::{DecodePublicKey, EncodePublicKey, LineEnding},
    Pkcs1v15Encrypt, RsaPublicKey,
};
use tokio::sync::Mutex;

use crate::{
    crypto::send_json,
    types::{AppState, ChatMsg, WsMessage},
    WsSink,
};

/// Deserialises a WebSocket text frame and dispatches to the correct handler.
pub async fn handle_ws_message(
    text: &str,
    state: &Arc<Mutex<AppState>>,
    write: &Arc<Mutex<WsSink>>,
) {
    let ws_msg: WsMessage = match serde_json::from_str(text) {
        Ok(m) => m,
        Err(_) => return,
    };

    match ws_msg {
        WsMessage::Token { token } => on_token(token, state, write).await,
        WsMessage::ClientList { clients } => on_client_list(clients, state).await,
        WsMessage::PublicKey { token, key } => on_public_key(token, key, state).await,
        WsMessage::KeyExchange { from, payload, .. } => on_key_exchange(from, payload, state).await,
        WsMessage::ChatMessage { from, n, data, .. } => on_chat_message(from, n, data, state).await,
    }
}

async fn on_token(token: String, state: &Arc<Mutex<AppState>>, write: &Arc<Mutex<WsSink>>) {
    let mut s = state.lock().await;
    s.token = token.clone();
    s.add_notification(format!("Connected as [{}]", token));

    if let Ok(pem) = s.public_key.to_public_key_pem(LineEnding::LF) {
        let msg = WsMessage::PublicKey { token, key: pem };
        drop(s); // release lock before awaiting send
        send_json(write, &msg).await;
    }
}

async fn on_client_list(clients: Vec<String>, state: &Arc<Mutex<AppState>>) {
    let mut s = state.lock().await;
    s.add_notification(format!("{} users online", clients.len()));
    s.online_users = clients;
}

async fn on_public_key(token: String, key: String, state: &Arc<Mutex<AppState>>) {
    let mut s = state.lock().await;
    if token == s.token {
        return;
    }
    if let Ok(pub_key) = RsaPublicKey::from_public_key_pem(&key) {
        s.peer_keys.insert(token.clone(), pub_key);
        s.add_notification(format!("Key from [{}]", token));
    }
}

async fn on_key_exchange(from: String, payload: String, state: &Arc<Mutex<AppState>>) {
    let mut s = state.lock().await;
    let private_key = s.private_key.clone();

    let Ok(encrypted) = general_purpose::STANDARD.decode(&payload) else { return };
    let Ok(decrypted) = private_key.decrypt(Pkcs1v15Encrypt, &encrypted) else { return };

    if decrypted.len() >= 44 {
        let aes_key = decrypted[0..32].to_vec();
        let nonce_seed = decrypted[32..44].to_vec();
        s.aes_keys.insert(from.clone(), (aes_key, nonce_seed, 0));
        s.add_notification(format!("Secure channel with [{}]", from));
    }
}

async fn on_chat_message(from: String, n: u64, data: String, state: &Arc<Mutex<AppState>>) {
    let mut s = state.lock().await;

    let Some((aes_key, nonce_seed, _)) = s.aes_keys.get(&from).cloned() else { return };

    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&aes_key));

    let mut nonce_bytes = nonce_seed;
    nonce_bytes.extend_from_slice(&n.to_le_bytes()[0..4]);
    // TODO: same as in crypto::send_encrypted_message - counter bytes are discarded here.
    let nonce = Nonce::from_slice(&nonce_bytes[0..12]);

    let Ok(ciphertext) = general_purpose::STANDARD.decode(&data) else { return };
    let Ok(plaintext) = cipher.decrypt(nonce, ciphertext.as_ref()) else { return };
    let Ok(message) = String::from_utf8(plaintext) else { return };

    let timestamp = Local::now().format("%H:%M").to_string();
    s.messages.push(ChatMsg {
        content: message,
        is_own: false,
        timestamp,
        sender: from.clone(),
    });

    if let Some((_, _, counter)) = s.aes_keys.get_mut(&from) {
        *counter = n + 1;
    }

    s.scroll_offset = 0;
}
