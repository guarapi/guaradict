use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
// use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::time::{Duration, Instant};

use super::ReplicaStatus;

pub struct ReplicaMonitorServer {
    replicas: Arc<Mutex<HashMap<String, ReplicaStatus>>>,
}

impl ReplicaMonitorServer {
    pub fn new(replicas: HashMap<String, ReplicaStatus>) -> Self {
        Self {
            replicas: Arc::new(Mutex::new(replicas)),
        }
    }

    pub async fn get_replica_status(&self, name: String) -> Option<ReplicaStatus> {
        let replicas = Arc::clone(&self.replicas);
        let mut replicas = replicas.as_ref().lock().await;
        let replica = replicas.get_mut(&name.to_string()).cloned();

        drop(replicas);

        replica
    }

    pub async fn start(&self) {
        let replicas = Arc::clone(&self.replicas);
        // Executa loop a cada 5 segundos
        let mut interval = tokio::time::interval(Duration::from_secs(5));

        loop {
            interval.tick().await;

            let mut replicas = replicas.as_ref().lock().await;

            for (_, replica) in replicas.iter_mut() {
                if let Some(addr) = &replica.addr {
                    let start_time = Instant::now();
                    // Tenta conectar com timeout de 3s
                    match connect_with_timeout(addr, Duration::from_secs(3)).await {
                        Ok(_) => {
                            let end_time = Instant::now();
                            replica.ping = end_time.duration_since(start_time);
                            replica.ready = true;
                        }
                        Err(_) => {
                            replica.ready = false;
                        }
                    }
                }
            }

            println!("{:?}", replicas);

            drop(replicas);
        }
    }
}

async fn connect_with_timeout(addr: &SocketAddr, timeout_duration: Duration) -> Result<TcpStream, std::io::Error> {
    tokio::time::timeout(timeout_duration, TcpStream::connect(addr)).await.unwrap()
}
