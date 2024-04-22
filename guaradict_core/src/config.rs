use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::io::BufReader;

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub primary: Option<Vec<Node>>,
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
    Ok(config)
}
