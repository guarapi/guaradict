use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use serde_yaml::{self, Value};
use regex::Regex;

use crate::errors::ConfigFileError;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub node_type: String,
    pub name: String,
    pub ip: String,
    pub host: String,
    pub port: u16,
    pub database: Option<String>,
    pub journal: Journal,
    pub replicas: Option<Vec<Replica>>,
}

impl Config {
    pub fn to_yaml_value(&self) -> Value {
        serde_yaml::to_value(&self).unwrap()
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Replica {
    pub node_type: String,
    pub name: String,
    pub ip: String,
    pub host: String,
    pub port: u16,
    pub database: Option<String>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Journal {
    pub strategy: String,
    pub size: u16,
}

pub fn parse_config_file(file_path: &str) -> Result<Config, ConfigFileError> {
    let file = File::open(file_path).map_err(ConfigFileError::IOError)?;
    let reader = BufReader::new(file);
    let config: Config = serde_yaml::from_reader(reader).map_err(ConfigFileError::YamlError)?;

    match validate_config(&config.to_yaml_value()) {
        Ok(()) => Ok(config),
        Err(e) => Err(ConfigFileError::ValidationError(e.to_string())),
    }
}

fn is_valid_ip(ip: &str) -> bool {
    // Verifica se a string pode ser convertida para um endereço IP
    match ip.parse::<std::net::IpAddr>() {
        Ok(_) => true,
        Err(_) => false,
    }
}

fn validate_database_name(database_name: &str) -> bool {
    let pattern = "^[a-z0-9-]+$"; // Padrão da regex
    let regex = Regex::new(pattern).unwrap();
    regex.is_match(database_name)
}

pub fn validate_config(config: &Value) -> Result<(), Box<dyn Error>> {
    let config = config.as_mapping().ok_or("Configuração YAML inválida")?;

    // Verifica se todas as chaves obrigatórias estão presentes
    let required_keys = ["nodeType", "name", "ip", "host", "port"];
    for key in &required_keys {
        if !config.contains_key(&Value::String(key.to_string())) {
            return Err(format!("Chave obrigatória ausente: {}", key).into());
        }
    }

    let node_type = config
        .get(&Value::String("nodeType".into()))
        .and_then(|value| value.as_str())
        .ok_or("Valor de nodeType inválido. Valores permitidos: 'primary', 'replica'")?;

    // Verifica se o tipo de nó é "primary" ou "replica"
    let valid_node_types = ["primary", "replica"];
    if !valid_node_types.contains(&node_type) {
        return Err("Valor de nodeType inválido. Valores permitidos: 'primary', 'replica'".into());
    }

    // Verifica se o valor do 'ip' é um IP válido
    let ip_str = config
        .get(&Value::String("ip".into()))
        .and_then(|value| value.as_str())
        .ok_or("Valor de IP inválido")?;
    if !is_valid_ip(ip_str) {
        return Err("Valor de IP inválido".into());
    }

    // Verifica se o 'port' está dentro do intervalo válido
    let port = config
        .get(&Value::String("port".to_string()))
        .and_then(|value| value.as_i64())
        .ok_or("Valor de porta inválido")?;
    if port < 1 || port > 65535 {
        return Err("Valor de porta inválido. A porta deve estar entre 1 e 65535".into());
    }

    // Verifica se o nome da base de dados está no formato correto, se presente
    if let Some(database_name) = config.get(&Value::String("database".into())) {
        if !database_name.is_null() {
            let database_name_str = database_name
                .as_str()
                .ok_or("Formato de nome de base de dados inválido")?;
            if !validate_database_name(database_name_str) {
                return Err("Formato de nome de base de dados inválido".into());
            }
        }
    }

    // Verifica as réplicas e o journal, se presentes
    let replicas = config.get(&Value::String("replicas".into()));

    if let Some(replicas) = replicas {
        if !replicas.is_null() {
            let replicas_array = replicas
                .as_sequence()
                .ok_or("Formato de réplicas inválido")?;
            for replica in replicas_array {
                if replica.get(&Value::String("journal".into())).is_some() {
                    return Err("O campo 'journal' não é permitido dentro do campo 'replicas'".into());
                }
                validate_config(replica)?;
            }
        }

        // Se não houver "replicas", verifica se "journal" está presente e valida suas configurações
        let journal = config
            .get(&Value::String("journal".into()))
            .ok_or("Chave obrigatória ausente: journal")?;
        let journal_mapping = journal
            .as_mapping()
            .ok_or("Configuração de journal inválida")?;
        let strategy = journal_mapping
            .get(&Value::String("strategy".into()))
            .ok_or("Estratégia de journal ausente")?;
        let size = journal_mapping
            .get(&Value::String("size".into()))
            .ok_or("Tamanho de journal ausente")?;

        let strategy_str = strategy
            .as_str()
            .ok_or("Estratégia de journal inválida")?;
        if !["async", "sync", "snapshot_log"].contains(&strategy_str) {
            return Err("Valor de estratégia de journal inválido. Valores permitidos: 'async', 'sync', 'snapshot_log'".into());
        }

        let size_int = size
            .as_i64()
            .ok_or("Tamanho de journal inválido")?;
        if size_int < 0 {
            return Err("Valor de tamanho de journal inválido. Deve ser um inteiro não negativo".into());
        }
    }

    Ok(())
}
