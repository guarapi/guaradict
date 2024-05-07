use std::sync::Arc;
use tokio::sync::mpsc::Receiver;
use tokio::sync::Mutex;

use crate::commands::client;
use crate::replica::log_operator::LogOperator;

pub struct SynchronizerServer {
    rx: Arc<Mutex<Receiver<client::Command>>>,
    operations_log: Arc<Mutex<LogOperator>>,
}

impl SynchronizerServer {
    pub fn new(rx: Receiver<client::Command>, operations_log: LogOperator) -> Self {
        Self {
            rx: Arc::new(Mutex::new(rx)),
            operations_log: Arc::new(Mutex::new(operations_log)),
        }
    }

    pub async fn start(&self) {
        let mut rx = self.rx.as_ref().lock().await;

        while let Some(log) = rx.recv().await {
            let mut op = self.operations_log.as_ref().lock().await;
            match log {
                client::Command::Add(key, value) => {
                    op.insert(key, value);
                    drop(op);
                },
                client::Command::Set(key, new_value) => {
                    // @TODO valor antigo do dicioario com enum proprio de log de operações
                    let prev_value = None;
                    op.update(key, new_value, prev_value);
                    drop(op);
                },
                client::Command::Del(key) => {
                    op.delete(key);
                    drop(op);
                },
                client::Command::Get(_) => {
                    println!("{:#?}", op);
                    drop(op);
                },
                client::Command::Quit => {},
            }
        }
    }
}
