pub enum Command {
    Ping(String),
    Pong(String),
}

impl Command {
    pub fn serialize(&self) -> String {
        match self {
            Command::Ping(replica_name) => format!("PING {}", replica_name),
            Command::Pong(replica_name) => format!("PONG {}", replica_name),
        }
    }


    pub fn parse(input: &str) -> Result<Command, &'static str> {
        let parts: Vec<&str> = input.trim().split_whitespace().collect();
        match parts.as_slice() {
            ["PING", replica_name] => Ok(Command::Ping(replica_name.to_string())),
            ["PONG", replica_name] => Ok(Command::Pong(replica_name.to_string())),
            _ => Err("Comando inválido"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execute_ping() {
        let command = Command::Ping("replica-node-1".into());
        assert_eq!(command.serialize(), "PING replica-node-1");
    }

    #[test]
    fn test_execute_pong() {
        let command = Command::Pong("primary-node".into());
        assert_eq!(command.serialize(), "PONG primary-node");
    }
}
