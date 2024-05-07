use std::net::SocketAddr;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplicaStatus {
    pub name: String,
    #[serde(with = "duration_serde")]
    pub ping: Duration,
    pub ready: bool,
    pub addr: Option<SocketAddr>,
    pub failures: u32,
    #[serde(skip)]
    pub(crate) stream: Option<Arc<Mutex<TcpStream>>>,
}

impl ReplicaStatus {
    pub fn new(name: String, ready: bool, addr: Option<SocketAddr>) -> Self {
        Self {
            name,
            ready,
            addr,
            ..Self::default()
        }
    }

    pub fn default() -> Self {
        Self {
            name: String::new(),
            ping: Duration::default(),
            failures: 0,
            ready: false,
            addr: None,
            stream: None,
        }
    }
}

impl Default for ReplicaStatus {
    fn default() -> Self {
        Self::default()
    }
}

mod duration_serde {
    use serde::{self, Serializer, Deserializer, Deserialize};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u64(duration.as_secs())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secs = u64::deserialize(deserializer)?;
        Ok(Duration::from_secs(secs))
    }
}
