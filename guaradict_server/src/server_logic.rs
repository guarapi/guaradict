use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;
use tokio::sync::Mutex;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use guaradict_core::{commands::client, Dictionary};

pub struct ServerLogic {
    tx: Arc<Mutex<Sender<client::Command>>>,
    dictionary: Arc<Mutex<Dictionary>>,
}

impl ServerLogic {
    pub fn new(tx: Sender<client::Command>, dictionary: Dictionary) -> Self {
        Self {
            tx: Arc::new(Mutex::new(tx)),
            dictionary: Arc::new(Mutex::new(dictionary)),
        }
    }

    pub async fn start(&self, listener: TcpListener) {
        println!("Servidor ouvindo em {:?}", listener.local_addr());
        let tx = self.tx.as_ref().lock().await;
        let dictionary = self.dictionary.as_ref().lock().await;

        // Loop principal para lidar com conexões de clientes
        loop {
            let (socket, _) = listener.accept().await.unwrap();

            println!("Nova conexão {} {}", socket.peer_addr().unwrap().ip(), socket.peer_addr().unwrap().port());

            let dictionary = dictionary.clone();
            let tx = tx.clone();

            // Lidar com o cliente em uma nova tarefa
            tokio::spawn(async move {
                if let Err(e) = Self::handle_client(socket, dictionary, tx).await {
                    eprintln!("Error handling client: {}", e);
                }
            });
        }
    }

    async fn handle_client(mut socket: TcpStream, dictionary: Dictionary, tx: Sender<client::Command>) -> Result<(), Box<dyn std::error::Error>> {
        let mut buffer = [0; 1024];

        while let Ok(n) = socket.read(&mut buffer).await {
            let request = String::from_utf8_lossy(&buffer[..n]);

            if n == 0 {
                continue;
            }

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

            // @TODO Refatorar usar frame com header e payload
            // @TODO Refatorar para não responder ping de replica com "Invalid command"
            let tx = tx.clone();
            let dictionary = dictionary.clone();
            let response = match client::Command::parse(request.trim()) {
                Ok(command) => {
                    match command {
                        client::Command::Add(key, value) => Self::add_entry(key, value, dictionary, tx).await,
                        client::Command::Set(key, value) => Self::add_entry(key, value, dictionary, tx).await,
                        client::Command::Get(key) => Self::get_definition(key, dictionary, tx).await,
                        client::Command::Del(key) => Self::remove_entry(key, dictionary, tx).await,
                        _ => "Invalid command".to_string(),
                    }
                },
                Err(_) => "Invalid command".to_string(),
            };

            if let Err(e) = socket.write_all(response.as_bytes()).await {
                eprintln!("Falha na resposta: {}", e);
            } else {
                // Log da resposta enviada
                println!("Response sent: {}", response);
            }
        }

        Ok(())
    }

    // @TODO add deve verificar se existe antes, set deve ser o update
    async fn add_entry(key: String, value: String, mut dictionary: Dictionary, tx: Sender<client::Command>) -> String {
        dictionary.add_entry(key.to_string(), value.to_string());

        match dictionary.get_definition(&key) {
            Some(_prev_value) => {
                // @TODO cria enum propriio para log de operaçoes para substituir o client::Command
                match tx.send(client::Command::Set(key.to_string(), value.to_string())).await {
                    Ok(_) => {},
                    Err(e) => {
                        println!("Erro Command::Add: {}", e)
                    }
                };
            },
            None => {
                match tx.send(client::Command::Add(key.to_string(), value.to_string())).await {
                    Ok(_) => {},
                    Err(e) => {
                        println!("Erro Command::Add: {}", e)
                    }
                };
            }
        }

        drop(dictionary);


        "Entry added successfully".to_string()
    }

    async fn get_definition(key: String, dictionary: Dictionary, tx: Sender<client::Command>) -> String {
        let result = match dictionary.get_definition(&key) {
            Some(definition) => format!("Definition: {}", definition),
            None => "Key not found".to_string(),
        };

        drop(dictionary);

        match tx.send(client::Command::Get(key.to_string())).await {
            Ok(_) => {},
            Err(e) => {
                println!("Erro Command::Get: {}", e)
            }
        };

        result
    }

    async fn remove_entry(key: String, mut dictionary: Dictionary, tx: Sender<client::Command>) -> String {
        dictionary.remove_entry(&key);

        drop(dictionary);

        match tx.send(client::Command::Del(key.to_string())).await {
            Ok(_) => {},
            Err(e) => {
                println!("Erro Command::Del: {}", e)
            }
        };

        "Entry removed successfully".to_string()
    }

}
