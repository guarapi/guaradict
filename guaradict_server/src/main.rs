use std::env;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use guaradict_core::dictionary::Dictionary;
use guaradict_core::config::{parse_config_file, Config};
use guaradict_core::commands::Command;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dictionary = Arc::new(Mutex::new(Dictionary::new()));

    let args: Vec<String> = env::args().collect();
    let mut config_file = "guaradict.yaml"; // Caminho padrão do arquivo de configuração

    // Verificar se o argumento --config foi fornecido
    if args.len() > 2 {
        if let Some(index) = args.iter().position(|arg| arg == "--config") {
            if index + 1 < args.len() {
                config_file = &args[index + 1];
            }
        }
    }

    println!("Carregando arquivo: {}", config_file);

    let config = match parse_config_file(config_file) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Error parsing config file: {}", e);
            return Err(e.into());
        }
    };

    let addr = format!("{}:{}", config.ip, config.port);
    let listener = TcpListener::bind(&addr).await?;
    println!("Servidor ouvindo em {}", addr);

    // Clonando os recursos necessários para a thread assíncrona
    let dictionary_clone = dictionary.clone();
    let config_clone = config.clone();

    // Gerenciador de réplica
    if let Some(replicas) = config.replicas.as_ref() {
        for replica in replicas {
            println!("Réplica: {} {}:{}", replica.name.clone(), replica.ip.clone(), replica.port);
        }
    } else {
        println!("Nenhuma réplica encontrada na configuração.");
    }

    // Thread assíncrona que gerencia a lógica do servidor
    let server_task = tokio::spawn(async move {
        if config_clone.node_type == "primary" {
            let replicas_ready = pool_replicas(&config_clone).await.unwrap();

            if replicas_ready {
                println!("Todas as réplicas estão prontas. Iniciando handshake...");

                // Inicia o handshake com as réplicas
                for replica in config_clone.replicas.as_ref().unwrap() {
                    handshake_with_replica(replica).await.unwrap();
                }

                println!("Handshake concluído com todas as réplicas");

                // Spawna uma thread para mandar delta a cada 60 segundos para todas as réplicas
                for replica in config_clone.replicas.as_ref().unwrap() {
                    let addr = format!("{}:{}", replica.ip, replica.port);
                    let dictionary_clone = dictionary_clone.clone();
                    if let Err(e) = send_delta_periodically(addr, dictionary_clone).await {
                        eprintln!("Error sending delta to replica {}: {}", replica.name, e);
                    }
                }
            }
        } else {
            println!("Servidor replica em standby...");
        }

        // Loop principal para lidar com conexões de clientes
        loop {
            let (socket, _) = listener.accept().await.unwrap();

            let dictionary_clone = dictionary.clone();

            if let Err(e) = handle_client(socket, dictionary_clone).await {
                eprintln!("Error handling client: {}", e);
            }
        }
    });

    // Aguardando o término do servidor
    server_task.await?;

    Ok(())
}

async fn pool_replicas(config: &Config) -> Result<bool, Box<dyn std::error::Error>> {
    println!("Pooling nas réplicas...");

    let mut replicas_ready = vec![false; config.replicas.as_ref().unwrap().len()];

    loop {
        for (index, replica) in config.replicas.as_ref().unwrap().iter().enumerate() {
            if replicas_ready[index] {
                continue;
            }

            if let Ok(mut _socket) = TcpStream::connect(format!("{}:{}", replica.ip, replica.port)).await {
                println!("Réplica {} está pronta", replica.name);
                replicas_ready[index] = true;
            } else {
                eprintln!("Failed to connect to replica {}", replica.name);
                // Tentar reconectar à réplica após um curto período de tempo
                let reconnect_delay = Duration::from_secs(5);
                tokio::time::sleep(reconnect_delay).await;
            }
        }

        if replicas_ready.iter().all(|&ready| ready) {
            println!("Todas as réplicas estão prontas");
            break;
        }

        // Aguarda um curto período antes de verificar novamente
        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    Ok(true)
}


async fn handshake_with_replica(replica: &guaradict_core::config::Replica) -> Result<(), Box<dyn std::error::Error>> {
    println!("Iniciando handshake com a réplica {}", replica.name);

    let mut socket = TcpStream::connect(format!("{}:{}", replica.ip, replica.port)).await?;
    socket.write_all(b"READY").await?;

    let mut buffer = [0; 5];
    socket.read_exact(&mut buffer).await?;

    if &buffer == b"READY" {
        println!("Handshake com a réplica {} concluído", replica.name);
    } else {
        println!("Erro no handshake com a réplica {}", replica.name);
    }

    Ok(())
}

async fn handle_client(mut socket: TcpStream, dictionary: Arc<Mutex<Dictionary>>) -> Result<(), Box<dyn std::error::Error>> {
    let mut buffer = [0; 1024];

    while let Ok(n) = socket.read(&mut buffer).await {
        let request = String::from_utf8_lossy(&buffer[..n]);

        if request.trim() == "QUIT" {
            // Fechar a conexão com o cliente e sair da função
            return Ok(());
        }

        let response = match Command::parse(request.trim()) {
            Ok(command) => match command {
                Command::Add(key, value) => add_entry(key, value, &dictionary),
                Command::Get(key) => get_definition(key, &dictionary),
                Command::Delete(key) => remove_entry(key, &dictionary),
                _ => "Invalid command".to_string(),
            },
            Err(_) => "Invalid command".to_string(),
        };

        if let Err(e) = socket.write_all(response.as_bytes()).await {
            eprintln!("Failed to send response: {}", e);
        } else {
            // Log da resposta enviada
            println!("Response sent: {}", response);
        }
    }

    Ok(())
}

fn add_entry(key: String, value: String, dictionary: &Arc<Mutex<Dictionary>>) -> String {
    dictionary.lock().unwrap().add_entry(key, value);
    "Entry added successfully".to_string()
}

fn get_definition(key: String, dictionary: &Arc<Mutex<Dictionary>>) -> String {
    match dictionary.lock().unwrap().get_definition(&key) {
        Some(definition) => format!("Definition: {}", definition),
        None => "Key not found".to_string(),
    }
}

fn remove_entry(key: String, dictionary: &Arc<Mutex<Dictionary>>) -> String {
    dictionary.lock().unwrap().remove_entry(&key);
    "Entry removed successfully".to_string()
}

async fn send_delta_periodically(addr: String, _dictionary: Arc<Mutex<Dictionary>>) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        tokio::time::sleep(Duration::from_secs(10)).await;
        println!("Enviado delta e escrevendo no journal para {}", addr);
    }
}
