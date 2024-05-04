use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::net::TcpListener;
use guaradict_core::{Dictionary, ReplicaStatus};
use guaradict_core::config::parse_config_file;

mod replica_monitor;
mod replica_sync;
mod server_logic;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dictionary = Arc::new(Mutex::new(Dictionary::new()));
    let replica_statuses: Arc<Mutex<HashMap<String, ReplicaStatus>>> = Arc::new(Mutex::new(HashMap::new()));

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

    if let Some(replicas) = config.replicas {
        let replica_statuses_clone = Arc::clone(&replica_statuses);
        replica_monitor::start(replicas, replica_statuses_clone).await;
        // replica_sync::start(replica_statuses.clone(), dictionary.clone()).await;
    } else {
        println!("Nenhuma réplica encontrada na configuração.");
    }

    server_logic::start(listener, dictionary).await;

    Ok(())
}
