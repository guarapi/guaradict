use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::Duration;

use crate::replica::log_operator::LogOperator;

pub struct SynchronizerServer {
    operations_log: Arc<Mutex<LogOperator>>,
}

impl SynchronizerServer {
    pub fn new(operations_log: LogOperator) -> Self {
        Self {
            operations_log: Arc::new(Mutex::new(operations_log)),
        }
    }

    pub async fn start(&self) {
        let log = Arc::clone(&self.operations_log);

        // Executa loop a cada 5 segundos
        let mut interval = tokio::time::interval(Duration::from_millis(5000));

        loop {
            interval.tick().await;

            let log = log.lock().await;

            // Imprimir o log
            println!("{:#?}", log);

            // Drop do lock
            drop(log);
        }
    }
}
