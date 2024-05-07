use std::fmt;
use std::error::Error;


#[derive(Debug)]
pub enum ConfigFileError {
    IOError(std::io::Error),
    YamlError(serde_yaml::Error),
    ValidationError(String),
}

impl fmt::Display for ConfigFileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            ConfigFileError::IOError(ref err) => write!(f, "Erro de E/S: {}", err),
            ConfigFileError::YamlError(ref err) => write!(f, "Erro de análise YAML: {}", err),
            ConfigFileError::ValidationError(ref msg) => write!(f, "Erro de validação de configuração: {}", msg),
        }
    }
}

impl Error for ConfigFileError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match *self {
            ConfigFileError::IOError(ref err) => Some(err),
            ConfigFileError::YamlError(ref err) => Some(err),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub enum ConfigError {
    MissingNodeProp(String),
    MissingReplicaProp(String),
    DeserializeError(serde_yaml::Error),
    MissingNode,
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::MissingNodeProp(prop) => write!(f, "Propriedade ausente no nó: {}", prop),
            ConfigError::MissingReplicaProp(prop) => write!(f, "Propriedade ausente na réplica: {}", prop),
            ConfigError::DeserializeError(err) => write!(f, "Erro de desserialização: {}", err),
            ConfigError::MissingNode => write!(f, "Configuração do nó ausente"),
        }
    }
}

impl Error for ConfigError {}
