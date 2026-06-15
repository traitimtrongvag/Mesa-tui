use std::sync::Arc;

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Key, Nonce,
};
use base64::{engine::general_purpose, Engine as _};
use futures_util::SinkExt;
use rand::RngCore;
use rsa::Pkcs1v15Encrypt;
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::Message;

use crate::types::{AppState, WsMessage};
use crate::WsSink;

/// Encrypts `message` with the peer's AES-256-GCM session key and sends it.
/// Nonce is derived from the 12-byte nonce_seed stored in `aes_keys`.
/// Note: the counter is currently appended then truncated away ([0..12]), so the nonce
/// does not actually change per message. A correct implementation should use an 8-byte
/// seed and XOR or concatenate the 4-byte counter to produce a unique 12-byte nonce.
pub async fn send_encrypted_message(
    to: &str,
    message: &str,
    state: &mut AppState,
    write: &Arc<Mutex<WsSink>>,
) -> Result<(), String> {
    let (aes_key, nonce_seed, counter) = state
        .aes_keys
        .get(to)
        .ok_or_else(|| format!("No AES key for {}", to))?
        .clone();

    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&aes_key));

    let mut nonce_bytes = nonce_seed.clone();
    nonce_bytes.extend_from_slice(&counter.to_le_bytes()[0..4]);
    // TODO: slicing [0..12] discards the counter bytes; nonce_seed should be 8 bytes
    // so the counter occupies bytes 8..12 and actually varies per message.
    let nonce = Nonce::from_slice(&nonce_bytes[0..12]);

    let ciphertext = cipher
        .encrypt(nonce, message.as_bytes())
        .map_err(|_| format!("Failed to encrypt message for {}", to))?;

    let msg = WsMessage::ChatMessage {
        to: to.to_string(),
        from: state.token.clone(),
        n: counter,
        data: general_purpose::STANDARD.encode(&ciphertext),
    };

    if let Some((_, _, c)) = state.aes_keys.get_mut(to) {
        *c += 1;
    }

    send_json(write, &msg).await;
    Ok(())
}

/// Generates a fresh AES-256 key + nonce seed, encrypts them with the peer's
/// RSA public key, and sends a KeyExchange message.
pub async fn initiate_key_exchange(
    to: &str,
    my_token: &str,
    state: &mut AppState,
    write: &Arc<Mutex<WsSink>>,
) -> Result<(), String> {
    let peer_key = state
        .peer_keys
        .get(to)
        .ok_or_else(|| format!("No public key for {}", to))?
        .clone();

    let mut rng = rand::thread_rng();
    let mut aes_key = vec![0u8; 32];
    let mut nonce_seed = vec![0u8; 12];
    rng.fill_bytes(&mut aes_key);
    rng.fill_bytes(&mut nonce_seed);

    let mut payload = aes_key.clone();
    payload.extend_from_slice(&nonce_seed);

    let encrypted = peer_key
        .encrypt(&mut rng, Pkcs1v15Encrypt, &payload)
        .map_err(|_| format!("Failed to encrypt key for {}", to))?;

    let msg = WsMessage::KeyExchange {
        to: to.to_string(),
        from: my_token.to_string(),
        payload: general_purpose::STANDARD.encode(&encrypted),
    };

    state.aes_keys.insert(to.to_string(), (aes_key, nonce_seed, 0));

    send_json(write, &msg).await;
    Ok(())
}

/// Serialises `msg` to JSON and sends it over the WebSocket sink.
pub async fn send_json(write: &Arc<Mutex<WsSink>>, msg: &WsMessage) {
    if let Ok(json) = serde_json::to_string(msg) {
        let mut w = write.lock().await;
        let _ = w.send(Message::text(json)).await;
    }
}
