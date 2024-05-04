use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use guaradict_core::{commands::client, Dictionary};

pub async fn start(listener: TcpListener, dictionary: Arc<Mutex<Dictionary>>) {
    // Loop principal para lidar com conexões de clientes
    loop {
        let (socket, _) = listener.accept().await.unwrap();
        let dictionary = Arc::clone(&dictionary);

        println!("Nova conexão {} {}", socket.peer_addr().unwrap().ip(), socket.peer_addr().unwrap().port());

        // Lidar com o cliente em uma nova tarefa
        tokio::spawn(async move {
            if let Err(e) = handle_client(socket, dictionary).await {
                eprintln!("Error handling client: {}", e);
            }
        });
    }
}

async fn handle_client(mut socket: TcpStream, dictionary: Arc<Mutex<Dictionary>>) -> Result<(), Box<dyn std::error::Error>> {
    let mut buffer = [0; 1024];

    while let Ok(n) = socket.read(&mut buffer).await {
        let request = String::from_utf8_lossy(&buffer[..n]);
        let dictionary = Arc::clone(&dictionary);

        if request.trim() == "QUIT" {
            // Fechar a conexão com o cliente e sair da função
            return Ok(());
        }

        if request.trim() == "PING" {
            if let Err(e) = socket.write_all(b"PONG\n").await {
                eprintln!("Falhano PONG: {}", e);
            } else {
                println!("< PONG");
            }
            continue;
        }

        // @TODO Refatorar usar frame coom header e payload
        let response = match client::Command::parse(request.trim()) {
            Ok(command) => match command {
                client::Command::Add(key, value) => add_entry(key, value, dictionary).await,
                client::Command::Get(key) => get_definition(key, dictionary).await,
                client::Command::Delete(key) => remove_entry(key, dictionary).await,
                _ => "Invalid command".to_string(),
            },
            Err(_) => "Invalid command".to_string(),
        };

        if let Err(e) = socket.write_all(response.as_bytes()).await {
            eprintln!("Faalha na resposta: {}", e);
        } else {
            // Log da resposta enviada
            println!("Response sent: {}", response);
        }
    }

    Ok(())
}


async fn add_entry(key: String, value: String, dictionary: Arc<Mutex<Dictionary>>) -> String {
    let mut dictionary = dictionary.as_ref().lock().await;
    dictionary.add_entry(key, value);
    "Entry added successfully".to_string()
}

async fn get_definition(key: String, dictionary: Arc<Mutex<Dictionary>>) -> String {
    let dictionary = dictionary.as_ref().lock().await;
    match dictionary.get_definition(&key) {
        Some(definition) => format!("Definition: {}", definition),
        None => "Key not found".to_string(),
    }
}

async fn remove_entry(key: String, dictionary: Arc<Mutex<Dictionary>>) -> String {
    let mut dictionary = dictionary.as_ref().lock().await;
    dictionary.remove_entry(&key);
    "Entry removed successfully".to_string()
}
