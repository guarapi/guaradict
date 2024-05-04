use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
pub struct ReplicaStatus {
    pub name: String,
    pub ping: u16,
    pub is_ready: bool,
    pub active_socket: Option<Arc<Mutex<TcpStream>>>,
}

impl ReplicaStatus {
    pub fn new(&self, name: String, ping: u16, is_ready: bool, active_socket: Option<Arc<Mutex<TcpStream>>>) -> Self {
        Self {
            name,
            ping,
            is_ready,
            active_socket,
        }
    }
}
