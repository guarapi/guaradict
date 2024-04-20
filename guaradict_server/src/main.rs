use std::sync::{Arc, Mutex};
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt}; // Adicionando as importações necessárias
use guaradict_core::Dictionary;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dictionary = Arc::new(Mutex::new(Dictionary::new()));

    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("Server listening on port 8080");

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

        // Se não for QUIT, continua processando a solicitação
        let response = match request.trim() {
            req if req.starts_with("ADD") => add_entry(req, &dictionary),
            req if req.starts_with("GET") => get_definition(req, &dictionary),
            req if req.starts_with("DELETE") => remove_entry(req, &dictionary),
            _ => "Invalid command".to_string(),
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


fn add_entry(request: &str, dictionary: &Arc<Mutex<Dictionary>>) -> String {
    let mut parts = request.splitn(3, ' ');
    parts.next(); // Ignore the ADD command
    if let (Some(word), Some(definition)) = (parts.next(), parts.next()) {
        dictionary.lock().unwrap().add_entry(word.to_string(), definition.to_string());
        "Entry added successfully".to_string()
    } else {
        "Invalid request format".to_string()
    }
}

fn get_definition(request: &str, dictionary: &Arc<Mutex<Dictionary>>) -> String {
    let mut parts = request.split_whitespace();
    parts.next(); // Ignore the GET command
    if let Some(word) = parts.next() {
        if let Some(definition) = dictionary.lock().unwrap().get_definition(word) {
            format!("Definition: {}", definition)
        } else {
            "Word not found".to_string()
        }
    } else {
        "Invalid request format".to_string()
    }
}

fn remove_entry(request: &str, dictionary: &Arc<Mutex<Dictionary>>) -> String {
    let mut parts = request.split_whitespace();
    parts.next(); // Ignore the DELETE command
    if let Some(word) = parts.next() {
        dictionary.lock().unwrap().remove_entry(word);
        "Entry removed successfully".to_string()
    } else {
        "Invalid request format".to_string()
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

