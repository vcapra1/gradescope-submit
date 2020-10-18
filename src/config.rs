use std::{fs, path::Path};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub course: Course,
    pub assignment: Assignment,
}

#[derive(Deserialize)]
pub struct Course {
    pub id: u64,
    pub name: String,
    pub term: String,
}

#[derive(Deserialize)]
pub struct Assignment {
    pub id: u64,
    pub name: String,
    pub files: Vec<String>,
}

#[derive(Debug)]
pub enum ConfigError {
    ReadError,
    ParseError,
}

impl Config {
    /// Load the config from the specified TOML file.
    pub fn load<T: AsRef<Path>>(file: T) -> Result<Self, ConfigError> {
        /* Read the config file into memory */
        let content = match fs::read_to_string(file) {
            Ok(content) => Ok(content),
            Err(_) => Err(ConfigError::ReadError),
        }?;

        /* Parse TOML */
        match toml::from_str(&content) {
            Ok(config) => Ok(config),
            Err(_) => Err(ConfigError::ParseError),
        }
    }
}
