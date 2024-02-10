use std::path::PathBuf;
use anyhow::Result;
use serde_derive::{Serialize, Deserialize};


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub feeds: Vec<ConfigFeed>,
    pub database: ConfigDatabase
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConfigFeed {
    pub id: String,
    pub rss_url: String,
    pub webhook_url: String,
    pub interval: u64,
    pub message: String
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConfigDatabase {
    pub connection: String
}


impl Default for Config {
    fn default() -> Self {
        Config {
            feeds: vec![],
            database: ConfigDatabase { 
                connection: "postgres://postgres:postgres@postgres/postgres".to_owned() 
            }
        }
    }
}

impl Config {
    pub fn load(path: String) -> Result<Self> {
        let config: Self = confy::load_path(PathBuf::from(path).as_path())?;
        Ok(config)
    }
}