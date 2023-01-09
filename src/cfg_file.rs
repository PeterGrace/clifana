use std::collections::HashMap;
use std::path::PathBuf;
use config::{Config, ConfigError};
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
#[allow(unused)]
pub struct ServerRef {
    pub name: String,
    pub url: String
}

#[derive(Deserialize, Debug, Clone)]
#[allow(unused)]
pub struct QueryRef {
    pub name: String,
    pub query: String
}


#[derive(Deserialize, Clone)]
#[allow(unused)]
pub struct ConfigFile {
    pub log_level: u8,
    pub servers: Vec<ServerRef>,
    pub queries: Vec<QueryRef>
}

impl ConfigFile {
    pub fn new(path: Option<PathBuf>) -> Result<Self, ConfigError> {
        let config_file_path: String = match path {
            None => "config.toml".to_string(),
            Some(p) => p.display().to_string()
        };
        let s = Config::builder()
            .add_source(
                config::File::with_name(config_file_path.as_str())
            )
            .add_source(
                config::Environment::with_prefix("CLIFANA")
                    .try_parsing(true)
                    .separator("_")
                    .list_separator(",")
            )
            .build()?;
        Ok(s.try_deserialize()?)
    }
}