use std::env;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use guaradict_core::dictionary::Dictionary;
use guaradict_core::config::parse_config_file;
use guaradict_core::commands::Command;
use tokio::time;

// https://github.com/clap-rs/clap

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

    let _config = match parse_config_file(config_file) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Error parsing config file: {}", e);
            return Err(e.into());
        }
    };

    let addr = "127.0.0.1:8080";
    let listener = TcpListener::bind(&addr).await?;
    println!("Server listening on port XXXXX");


    // Spawna uma thread para mandar delta a cada 60 segundos
    tokio::spawn(send_delta_periodically(dictionary.clone()));

    loop {
        let (socket, _) = listener.accept().await?;
        let dictionary_clone = dictionary.clone();

        tokio::spawn(async move {
            if let Err(e) = handle_client(socket, dictionary_clone).await {
                eprintln!("Error handling client: {}", e);
            }
        });
    }
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

async fn send_delta_periodically(_dictionary: Arc<Mutex<Dictionary>>) {
    loop {
        time::sleep(Duration::from_secs(60)).await;
        println!("Enviado delta e escrevendo no journal...");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_invalid_request() {
        let mut dictionary = Dictionary::new();
        dictionary.add_entry("hello".to_string(), "a greeting".to_string());

        // Criando um servidor para simular a escuta de solicitações
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let server_addr = listener.local_addr().unwrap();

        // Armazena o endereço do servidor para reutilização
        let server_addr_clone = server_addr.clone();

        // Manipula a solicitação no lado do servidor
        let server_task = tokio::spawn(async move {
            let (socket, _) = listener.accept().await.unwrap();
            if let Err(e) = handle_client(socket, Arc::new(Mutex::new(dictionary))).await {
                eprintln!("Error handling client: {}", e);
            }
        });

        // Criando um cliente para simular a conexão ao servidor
        let mut client = TcpStream::connect(server_addr_clone).await.unwrap();

        // Escreve a solicitação inválida no lado do cliente
        let _ = tokio::spawn(async move {
            client.write_all(b"INVALID REQUEST").await.unwrap();
            drop(client); // Fechando explicitamente a conexão
        }).await;

        // Espera a finalização do servidor
        server_task.await.unwrap();
    }

    #[tokio::test]
    async fn test_get_definition() {
        let mut dictionary = Dictionary::new();
        dictionary.add_entry("hello".to_string(), "a greeting".to_string());

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let server_addr = listener.local_addr().unwrap();

        let server_task = tokio::spawn(async move {
            let (socket, _) = listener.accept().await.unwrap();
            if let Err(e) = handle_client(socket, Arc::new(Mutex::new(dictionary))).await {
                eprintln!("Error handling client: {}", e);
            }
        });

        let mut client = TcpStream::connect(server_addr).await.unwrap();

        let _ = tokio::spawn(async move {
            client.write_all(b"GET hello").await.unwrap();
            drop(client);
        }).await;

        server_task.await.unwrap();
    }

    #[tokio::test]
    async fn test_add_entry() {
        let dictionary = Dictionary::new();

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let server_addr = listener.local_addr().unwrap();

        let server_task = tokio::spawn(async move {
            let (socket, _) = listener.accept().await.unwrap();
            if let Err(e) = handle_client(socket, Arc::new(Mutex::new(dictionary))).await {
                eprintln!("Error handling client: {}", e);
            }
        });

        let mut client = TcpStream::connect(server_addr).await.unwrap();

        let _ = tokio::spawn(async move {
            client.write_all(b"ADD hello world").await.unwrap();
            drop(client);
        }).await;

        server_task.await.unwrap();
    }

    #[tokio::test]
    async fn test_remove_entry() {
        let mut dictionary = Dictionary::new();
        dictionary.add_entry("hello".to_string(), "a greeting".to_string());

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let server_addr = listener.local_addr().unwrap();

        let server_task = tokio::spawn(async move {
            let (socket, _) = listener.accept().await.unwrap();
            if let Err(e) = handle_client(socket, Arc::new(Mutex::new(dictionary))).await {
                eprintln!("Error handling client: {}", e);
            }
        });

        let mut client = TcpStream::connect(server_addr).await.unwrap();

        let _ = tokio::spawn(async move {
            client.write_all(b"DELETE hello").await.unwrap();
            drop(client);
        }).await;

        server_task.await.unwrap();
    }

    #[tokio::test]
    async fn test_quit() {
        let dictionary = Dictionary::new();

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let server_addr = listener.local_addr().unwrap();

        let server_task = tokio::spawn(async move {
            let (socket, _) = listener.accept().await.unwrap();
            if let Err(e) = handle_client(socket, Arc::new(Mutex::new(dictionary))).await {
                eprintln!("Error handling client: {}", e);
            }
        });

        let mut client = TcpStream::connect(server_addr).await.unwrap();

        let _ = tokio::spawn(async move {
            client.write_all(b"QUIT").await.unwrap();
            drop(client);
        }).await;

        server_task.await.unwrap();
    }
}
