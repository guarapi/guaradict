use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
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

    async fn connect_with_timeout(&self, addr: &SocketAddr, timeout_duration: Duration) -> Result<TcpStream, std::io::Error> {
        tokio::time::timeout(timeout_duration, TcpStream::connect(addr)).await.unwrap()
    }

    async fn heartbeat(&self, stream: &mut TcpStream, timeout_duration: Duration) -> Result<Duration, std::io::Error> {
        let start_time = Instant::now();
        stream.write_all(b"PING\n").await?;
        let mut buf = [0; 5];
        tokio::time::timeout(timeout_duration, stream.read_exact(&mut buf)).await??;
        if &buf == b"PONG\n" {
            Ok(start_time.elapsed())
        } else {
            println!("RESPOSTA NAO FOI PONG: ({})", String::from_utf8_lossy(&buf[..5]));
            Err(std::io::Error::new(std::io::ErrorKind::Other, "Resposta inválida"))
        }
    }

    pub async fn start(&self) {
        let replicas = Arc::clone(&self.replicas);
        let mut interval = tokio::time::interval(Duration::from_secs(1));

        loop {
            interval.tick().await;

            let mut replicas = replicas.as_ref().lock().await;

            for (_, replica) in replicas.iter_mut() {
                if let Some(addr) = &replica.addr {
                    if let Some(stream) = &mut replica.stream {
                        let mut locked_stream = stream.lock().await;
                        match self.heartbeat(&mut *locked_stream, Duration::from_secs(1)).await {
                            Ok(ping_time) => {
                                replica.ping = ping_time;
                                replica.ready = true;
                                replica.failures = 0;
                                println!("Sucesso no PING");
                            }
                            Err(e) => {
                                replica.ping = Duration::default();
                                replica.ready = false;
                                replica.failures += 1;
                                if replica.failures >= 3 {
                                    drop(locked_stream);
                                    replica.stream = None;
                                }
                                println!("Erro no PING: {}", e);
                            }
                        }
                    } else {
                        match self.connect_with_timeout(addr, Duration::from_secs(3)).await {
                            Ok(stream) => {
                                replica.stream = Some(Arc::new(Mutex::new(stream)));
                                replica.ping = Duration::default();
                                replica.ready = true;
                                replica.failures = 0;
                                println!("Sucesso na reconexão");
                            }
                            Err(e) => {
                                replica.ping = Duration::default();
                                replica.ready = false;
                                replica.failures += 1;
                                if replica.failures >= 3 {
                                    replica.stream = None;
                                }
                                println!("Erro ao reconectar: {}", e);
                            }
                        }
                    }
                }
            }

            println!("{:#?}", replicas);

            drop(replicas);
        }
    }
}
