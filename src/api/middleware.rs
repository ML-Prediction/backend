use std::sync::{Arc, Mutex};
use rusqlite::Connection;

#[derive(Clone)]
pub struct AuthState {
    pub conn: Arc<Mutex<Connection>>,
    pub secret: String,
}

impl AuthState {
    pub fn new(conn: Connection, _secret: String) -> Self {
        AuthState {
            conn: Arc::new(Mutex::new(conn)),
            secret: String::new(), // Não usado mais, mas mantido para compatibilidade
        }
    }
}

