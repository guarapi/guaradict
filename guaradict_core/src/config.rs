use serde::{Deserialize, Serialize};
use std::fmt;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use serde_yaml;

#[derive(Debug)]
pub enum ConfigError {
    PrimaryReplicaConflict,
    MissingPrimaryOrReplica,
    MissingJournalStrategy,
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::PrimaryReplicaConflict => write!(f, "Configuração contém conflito entre 'primary' e 'replica'"),
            ConfigError::MissingPrimaryOrReplica => write!(f, "Configuração deve conter pelo menos 'primary' ou 'replica'"),
            ConfigError::MissingJournalStrategy => write!(f, "Configuração de 'primary' não contém estratégia de jornal"),
        }
    }
}

impl Error for ConfigError {}

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub primary: Option<Vec<Node>>,
    pub replica: Option<Vec<Node>>,
    pub journal: Option<Journal>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Node {
    pub name: String,
    pub ip: String,
    pub host: String,
    pub database: String,
    pub replicas: Option<Vec<Replica>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Replica {
    pub name: String,
    pub ip: String,
    pub host: String,
    pub database: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Journal {
    pub strategy: String,
}

pub fn parse_config_file(file_path: &str) -> Result<Config, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let config: Config = serde_yaml::from_reader(reader)?;

    // Verifica se há conflito entre 'primary' e 'replica'
    if config.primary.is_some() && config.replica.is_some() {
        return Err(Box::new(ConfigError::PrimaryReplicaConflict));
    }

    // Verifica se há pelo menos 'primary' ou 'replica'
    if config.primary.is_none() && config.replica.is_none() {
        return Err(Box::new(ConfigError::MissingPrimaryOrReplica));
    }

    // Verifica se há estratégia de jornal em 'primary'
    if config.journal.is_none() || config.journal.as_ref().unwrap().strategy.is_empty() {
        return Err(Box::new(ConfigError::MissingJournalStrategy));
    }

    Ok(config)
}
