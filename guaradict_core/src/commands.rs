pub enum Command {
    Get(String),
    Add(String, String),
    Delete(String),
    QUIT,
}

impl Command {
    pub fn execute(&self) -> String {
        match self {
            Command::Get(key) => format!("GET {}", key),
            Command::Add(key, value) => format!("ADD {} {}", key, value),
            Command::Delete(key) => format!("DELETE {}", key),
            Command::QUIT => format!("QUIT"),
        }
    }


    pub fn parse(input: &str) -> Result<Command, &'static str> {
        let parts: Vec<&str> = input.trim().split_whitespace().collect();
        match parts.as_slice() {
            ["GET", key] => Ok(Command::Get(key.to_string())),
            ["DELETE", key] => Ok(Command::Delete(key.to_string())),
            ["ADD", key, value @ ..] => Ok(Command::Add(key.to_string(), value.join(" "))),
            ["QUIT"] => Ok(Command::QUIT),
            _ => Err("Comando inválido"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execute_get() {
        let command = Command::Get("key1".to_string());
        assert_eq!(command.execute(), "GET key1");
    }

    #[test]
    fn test_execute_add() {
        let command = Command::Add("key1".to_string(), "value1".to_string());
        assert_eq!(command.execute(), "ADD key1 value1");
    }

    #[test]
    fn test_execute_delete() {
        let command = Command::Delete("key1".to_string());
        assert_eq!(command.execute(), "DELETE key1");
    }
}
