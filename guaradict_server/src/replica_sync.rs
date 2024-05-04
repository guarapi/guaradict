use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::sleep;
use guaradict_core::{Dictionary, ReplicaStatus};

pub async fn start(replica_statuses: Arc<Mutex<HashMap<String, ReplicaStatus>>>, _dictionary: Arc<Mutex<Dictionary>>) {
    loop {
        // Obtém uma cópia do mapa de status das réplicas
        let replica_statuses = replica_statuses.lock().unwrap();

        // Itera sobre cada réplica e executa a sincronização delta
        for (_, status) in replica_statuses.iter() {
            // Verifica se a réplica está pronta e se possui um socket ativo
            if status.is_ready && status.active_socket.is_some() {
                // Execute a lógica de sincronização delta usando o socket ativo da réplica
                // Você pode implementar essa lógica aqui
                // Exemplo: enviar atualizações delta para a réplica
            }
        }

        // Aguarda um intervalo de tempo antes de executar a próxima sincronização
        sleep(Duration::from_secs(60)).await;
    }
}
