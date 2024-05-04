use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::net::SocketAddr;
use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::time::{sleep, Duration};
use guaradict_core::{ReplicaStatus, config::Replica};

pub async fn start(replicas: Vec<Replica>, replica_statuses: Arc<Mutex<HashMap<String, ReplicaStatus>>>) {
    for replica in replicas {
        let replica_statuses_ping = Arc::clone(&replica_statuses);
        let replica_statuses_monitor = Arc::clone(&replica_statuses);
        let replica_name_ping = replica.name.to_string();
        let replica_name_monitor = replica.name.to_string();
        let replica_addr: SocketAddr = format!("{}:{}", replica.ip, replica.port).parse().unwrap();

        // Spawn para a tarefa de ping
        tokio::spawn(async move {
            ping(replica_name_ping, replica_statuses_ping).await;
        });

        // Spawn para a tarefa de replica_monitor
        tokio::spawn( async move {
            replica_monitor(replica_name_monitor, replica_addr, replica_statuses_monitor).await;
        });
    }
}

async fn ping(name: String, replica_statuses: Arc<Mutex<HashMap<String, ReplicaStatus>>>) {
    loop {
        let mut replica_statuses = replica_statuses.as_ref().lock().await;

        if let Some(status) = replica_statuses.get_mut(&name) {
            if let Some(socket) = &status.active_socket {
                let mut socket = socket.as_ref().lock().await;

                if let Err(_) = socket.write_all(b"PING\n").await {
                    eprintln!("Erro ennviando PONG para {}", name);
                }

                let mut buf = [0; 5];
                if let Err(_) = socket.read_exact(&mut buf).await {
                    eprintln!("Erro esperando PONG de {}", name);
                };

                let response = String::from_utf8_lossy(&buf);
                if response.trim() == "PONG" {
                    println!("> PONG");
                } else {
                    println!("Resposta incorreta: {}", response);
                }
            } else {
                eprintln!("Socket da réplica {} não está ativo", name);
            }
        } else {
            eprintln!("Réplica {} não encontrada", name);
        }

        sleep(Duration::from_secs(3)).await;
    }
}

async fn replica_monitor(name: String, addr: SocketAddr, replica_statuses: Arc<Mutex<HashMap<String, ReplicaStatus>>>) {
    loop {
        let replica_statuses = Arc::clone(&replica_statuses);

        match TcpStream::connect(&addr).await {
            Ok(stream) => {
                println!("Réplica {} está UP", name);
                update_replica_status(&name, true, Some(Arc::new(Mutex::new(stream))), replica_statuses).await;
            },
            Err(_) => {
                println!("Réplica {} está DOWN", name);
                update_replica_status(&name, false, None, replica_statuses).await;
            },
        }

        sleep(Duration::from_secs(10)).await;
    }
}

async fn update_replica_status(name: &str, is_ready: bool, active_socket: Option<Arc<Mutex<TcpStream>>>, replica_statuses: Arc<Mutex<HashMap<String, ReplicaStatus>>>) {
    let mut replica_statuses = replica_statuses.as_ref().lock().await;

    let status = replica_statuses.entry(name.to_string()).or_insert(ReplicaStatus {
        name: name.to_string(),
        ping: 0,
        is_ready: false,
        active_socket: None,
    });

    status.is_ready = is_ready;
    status.active_socket = active_socket;
}
