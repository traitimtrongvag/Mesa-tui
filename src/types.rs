use std::{collections::HashMap, time::Instant};

use rsa::{RsaPrivateKey, RsaPublicKey};
use serde::{Deserialize, Serialize};

use crate::constants::MAX_NOTIFICATION_COUNT;

// Wire protocol

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type")]
pub enum WsMessage {
    #[serde(rename = "pubkey")]
    PublicKey { token: String, key: String },

    #[serde(rename = "key_exchange")]
    KeyExchange { to: String, from: String, payload: String },

    #[serde(rename = "msg")]
    ChatMessage { to: String, from: String, n: u64, data: String },

    #[serde(rename = "token")]
    Token { token: String },

    #[serde(rename = "clients")]
    ClientList { clients: Vec<String> },
}

// Local message record

#[derive(Clone)]
pub struct ChatMsg {
    pub content: String,
    pub is_own: bool,
    pub timestamp: String,
    pub sender: String,
}

// Shared application state

pub struct AppState {
    pub messages: Vec<ChatMsg>,
    pub input: String,
    pub cursor_position: usize,
    pub token: String,
    pub online_users: Vec<String>,
    pub notifications: Vec<String>,
    pub scroll_offset: usize,
    pub last_input_time: Instant,
    pub paste_buffer: String,
    pub private_key: RsaPrivateKey,
    pub public_key: RsaPublicKey,
    pub peer_keys: HashMap<String, RsaPublicKey>,
    /// token -> (aes_key, nonce_seed, counter)
    pub aes_keys: HashMap<String, (Vec<u8>, Vec<u8>, u64)>,
    pub is_connected: bool,
}

impl AppState {
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();
        let private_key =
            RsaPrivateKey::new(&mut rng, 2048).expect("Failed to generate RSA key");
        let public_key = RsaPublicKey::from(&private_key);

        Self {
            messages: Vec::new(),
            input: String::new(),
            cursor_position: 0,
            token: String::new(),
            online_users: Vec::new(),
            notifications: Vec::new(),
            scroll_offset: 0,
            last_input_time: Instant::now(),
            paste_buffer: String::new(),
            private_key,
            public_key,
            peer_keys: HashMap::new(),
            aes_keys: HashMap::new(),
            is_connected: false,
        }
    }

    pub fn add_notification(&mut self, msg: String) {
        self.notifications.push(msg);
        if self.notifications.len() > MAX_NOTIFICATION_COUNT {
            self.notifications.remove(0);
        }
    }
}
